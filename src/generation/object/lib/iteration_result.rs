/// Expresses the result of an iteration of the wave function collapse. Both `Ok` and `Incomplete` are considered
/// successful iterations, while `Failure` indicates that the iteration could not be completed successfully.
#[derive(PartialEq)]
pub enum IterationResult {
  Ok,
  Incomplete,
  Failure,
}
