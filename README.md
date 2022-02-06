# git-issues
Tool for producing lists of issue references

This is a work-in-progress program for producing a list of Jira issue
references from git log. For any detected references, it is intended
to find corresponding issue Summaries (i.e. titles).

# ADRs

## Rust
Rust was chosen as the implementation language. The choice is not
entirely obvious, because of these drawbacks:

- Use of Rust involves a relatively heavy compilation step
- Type system of Rust is more complex than the problem really
  requires, but some attention needs to be paid to get e.g. ownership,
  lifetimes and generic types right.

Despite the drawbacks, Rust was chosen for these qualities:

- The resulting executables are reasonably simple to manage and they
  start up quickly
- Combined with this, a robust argument parser such as clap makes it
  easy to provide a comfortable command line user interface
- In addition to the technical convenience, an important reason is
  that writing a simple, but complete tool for something is a good way
  to get some more experience in Rust.

A good alternative - and arguably a better fit for basically pushing
bytes around - would have been Go. What turned the choice to Rust's
favour was that the author was already familiar with good options on
command line argument processing with Rust.
