# guessture

This library is a Rust implementation of the [$1 Unistroke Recognizer algorithm](https://depts.washington.edu/acelab/proj/dollar/index.html). Given a set of template gestures, this library can compare
2d paths of points to those templates and report how closely they match.

This library's API surface is small—it exposes types for recording path data (`Path2D`), storing
normalized gesture templates (`Template`), and matching a path aginst templates (`find_matching_template`/`find_matching_template_with_defaults`). Integration with user input toolkits is left to
other libraries as an exercise for the reader.
