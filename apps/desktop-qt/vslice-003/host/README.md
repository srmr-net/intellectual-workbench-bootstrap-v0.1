# wb-vslice003-worker product entry

This standalone binary links the accepted Rust APIs and preserves the desktop.1
and operator.1 wire shapes. See [desktop entry](../README.md) for startup arguments,
build commands, scope, workspace CAS and two-stage persistence/initialization.
Public library APIs are unchanged. Commands retain existing exact-pin single-use
consent; all guest execution occurs through Session then the accepted Desktop.
