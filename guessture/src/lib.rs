use euclid::Angle;
use euclid::default::{Box2D, Point2D};

const NUM_POINTS: usize = 64;
const SQUARE_SIZE: f32 = 250.0;

type PathCoord = f32;

#[derive(Default, Debug, Clone)]
pub struct Path2D {
    points: Vec<Point2D<PathCoord>>,
}

impl Path2D {
    pub fn points(&self) -> Vec<(PathCoord, PathCoord)> {
        self.points.iter().map(|p| (p.x, p.y)).collect()
    }

    pub fn push(&mut self, x: PathCoord, y: PathCoord) {
        self.points.push(Point2D::new(x, y));
    }

    pub fn is_new_point(&self, x: PathCoord, y: PathCoord) -> bool {
        let last = self.points.last();
        last.map_or(true, |last| *last != Point2D::new(x, y))
    }

    fn length(&self) -> PathCoord {
        let mut i = 1;
        let mut total: PathCoord = 0.0;
        while i < self.points.len() {
            total += self.points[i].distance_to(self.points[i - 1]);
            i += 1;
        }
        assert!(total >= 0.0);
        return total;
    }

    fn indicative_angle(&self) -> f32 {
        let centroid = self.centroid();
        return (centroid.y - self.points[0].y).atan2(centroid.x - self.points[0].x);
    }

    #[allow(non_snake_case)]
    fn resample(&self, num_points: usize) -> Path2D {
        let interval_length = self.length() / (num_points - 1) as PathCoord;
        let mut D: PathCoord = 0.0;

        let mut old_points = self.points.clone();

        let mut resampled = Path2D {
            points: vec![self.points[0]],
        };

        let mut i = 1;
        while i < old_points.len() {
            let d = old_points[i].distance_to(old_points[i - 1]);

            if D + d > interval_length {
                let qx = old_points[i - 1].x +
                    ((interval_length - D) / d) *
                    (old_points[i].x - old_points[i - 1].x);
                let qy = old_points[i - 1].y +
                    ((interval_length - D) / d) *
                    (old_points[i].y - old_points[i - 1].y);

                let point = Point2D::new(qx, qy);
                resampled.points.push(point);
                old_points.insert(i, point);
                D = 0.0;
            } else {
                D += d;
            }

            i += 1;
        }

        if resampled.points.len() == num_points - 1 {
            resampled.points.push(old_points[old_points.len() - 1]);
        }
        return resampled;
    }

    fn centroid(&self) -> Point2D<PathCoord> {
        let mut qx: PathCoord = 0.0;
        let mut qy: PathCoord = 0.0;

        for point in &self.points {
            qx += point.x;
            qy += point.y;
        }

        qx /= self.points.len() as PathCoord;
        qy /= self.points.len() as PathCoord;

        return Point2D::new(qx, qy);
    }

    fn rotate_by(&self, radians: f32) -> Path2D {
        let centroid = self.centroid();
        let (sin, cos) = radians.sin_cos();
        let mut rotated = Path2D {
            points: vec![]
        };

        for point in &self.points {
            let adjusted = *point - centroid;
            let qx = adjusted.x * cos -
                adjusted.y * sin +
                centroid.x;
            let qy = adjusted.x * sin +
                adjusted.y * cos +
                centroid.y;
            rotated.points.push(Point2D::new(qx, qy));
        }
        return rotated;
    }

    #[allow(non_snake_case)]
    fn scale_by(&self, size: f32) -> Path2D {
        let B = self.bounding_rect();
        let mut scaled = Path2D {
            points: vec![],
        };
        for point in &self.points {
            let qx = point.x * (size / B.width());
            let qy = point.y * (size / B.height());
            scaled.points.push(Point2D::new(qx, qy));
        }
        return scaled;
    }

    fn bounding_rect(&self) -> Box2D<PathCoord> {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        for point in &self.points {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }
        return Box2D::new(
            Point2D::new(min_x, min_y),
            Point2D::new(max_x, max_y),
        );
    }

    fn translate_to(&self, dest: Point2D<PathCoord>) -> Path2D {
        let centroid = self.centroid();
        let mut translated = Path2D {
            points: vec![],
        };

        for point in &self.points {
            let translated_point = *point + (dest - centroid);
            translated.points.push(translated_point);
        }

        return translated;
    }

    fn gss(&self, a: f32, b: f32, template: &Path2D) -> (f32, f32) {
        let phi = 0.5f32 * (-1.0 + 5.0f32.sqrt());
        let x = phi * a + (1.0 - phi) * b;
        return (
            x,
            self.distance_at_angle(template, x),
        );
    }

    fn distance_at_best_angle(
        &self,
        template: &Path2D,
        mut from_angle: f32,
        mut to_angle: f32,
        threshold: f32,
    ) -> f32 {
        let (mut x1, mut f1) = self.gss(from_angle, to_angle, template);
        let (mut x2, mut f2) = self.gss(to_angle, from_angle, template);

        while (to_angle - from_angle).abs() > threshold {
            if f1 < f2 {
                to_angle = x2;
                x2 = x1;
                f2 = f1;
                (x1, f1) = self.gss(from_angle, to_angle, template);
            } else {
                from_angle = x1;
                x1 = x2;
                f1 = f2;
                (x2, f2) = self.gss(to_angle, from_angle, template);
            }
        }
        return f1.min(f2);
    }

    fn distance_at_angle(&self, template: &Path2D, radians: f32) -> f32 {
        let rotated = self.rotate_by(radians);
        return rotated.path_distance(&template);
    }

    #[allow(non_snake_case)]
    fn path_distance(&self, other: &Path2D) -> f32 {
        if self.points.len() != other.points.len() {
            return f32::MAX;
        }
        let mut D = 0.0f32;
        for (point_a, point_b) in self.points.iter().zip(other.points.iter()) {
            D += point_b.distance_to(*point_a);
        }
        return D / self.points.len() as f32;
    }
}

pub struct Template {
    pub name: String,
    pub path: Path2D,
}

impl Template {
    pub fn new(name: String, points: Path2D) -> Result<Template, ()> {
        if points.points.is_empty() {
            return Err(());
        }

        let points = points.resample(NUM_POINTS);
        let radians = points.indicative_angle();
        let points = points.rotate_by(-radians);
        let points = points.scale_by(SQUARE_SIZE);
        let points = points.translate_to(Point2D::default());
        Ok(Template {
            name,
            path: points,
        })
    }

    pub fn new_raw(name: String, points: Path2D) -> Result<Template, ()> {
        if points.points.is_empty() {
            return Err(());
        }

        Ok(Template {
            name,
            path: points,
        })
    }
}

#[derive(Debug)]
pub enum Error {
    TooShort,
    NoMatch,
}

pub fn find_matching_template_with_defaults(
    templates: &[Template],
    path: Path2D,
) -> Result<(&Template, f32), Error> {
    return find_matching_template(templates, path, 45.0, 2.0);
}

pub fn find_matching_template(
    templates: &[Template],
    path: Path2D,
    angle_range: f32,
    angle_precision: f32,
) -> Result<(&Template, f32), Error> {
    if path.points.len() < 2 || path.length() < 100.0 {
        return Err(Error::TooShort);
    }

    let diagonal = (2.0f32 * SQUARE_SIZE * SQUARE_SIZE).sqrt();
    let half_diagonal = 0.5f32 * diagonal;

    let candidate = Template::new("".to_owned(), path).map_err(|()| Error::TooShort)?;

    let angle_range: f32 = Angle::degrees(angle_range).get();
    let angle_precision: f32 = Angle::degrees(angle_precision).get();
    let mut template_match = Err(Error::NoMatch);
    let mut best_distance = f32::MAX;
    for template in templates {
        let distance = candidate.path.distance_at_best_angle(
            &template.path,
            -angle_range,
            angle_range,
            angle_precision,
        );
        if distance < best_distance {
            best_distance = distance;
            template_match = Ok((template, 1.0 - best_distance / half_diagonal));
        }
    }
    return template_match;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
