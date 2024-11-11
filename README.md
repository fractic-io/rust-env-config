Rust utility to declare a program's execution requirements (ex. environment variables, secrets), and access them in a typed way. The configurations should be loaded at the start of the program, which will ensure all variables are present. Once loaded, the values are stored in a typed enum, such that the type system can ensure only valid values are used in the code.

Currently supports loading from:
- Environment variables.
- Secrets stored in AWS Secrets Manager.

This code is provided as-is. For the time being, attention will not be given to backwards compatibility or clear documentation. It is open-sourced mainly for the chance that snippets may be useful to others looking to do similar tasks. Eventually, this may become a real library productionized and documented for external use.
