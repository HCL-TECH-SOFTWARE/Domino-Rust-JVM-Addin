# Rust Addin Example

To deploy this addin:

1) Copy either "nrustaddin.exe" (for Windows servers) or "rustaddin" (for Linux servers) to the Domino program directory
2) Copy the "rustaddin-lib" directory and its contents to the Domino program directory

The "rustaddin-lib" directory can also be deployed elsewhere on the filesystem and referenced by setting the "RustAddinInstallDir" notes.ini parameter to point to it.

Once deployed, it can be run by executing `load rustaddin`, and then running `tell rustaddin <anything>` will echo what you send it back to the console.

## Environment Variables

The native addin uses a number of optional environment variables to control its behavior and the parameters of the JVM it spawns:

| Variable                      | Description |
|-------------------------------|-------------|
| RustAddinInstallDir           | The location containing the addin JARs. Defaults to "(Domino program dir)/rustaddin-lib" |
| RustAddinDebug                | "1" to enable some additional output from the native addin and include Rust backtraces |
| RustAddinJavaDebug            | "1" to enable remote JVM debugging |
| RustAddinJavaDebugCmd         | A debug configuration line to override the default of "-agentlib:jdwp=transport=dt_socket,server=y,address=0.0.0.0:8000,suspend=y" |
| RustAddinJavaHome             | The location of the JVM to run the addin. Defaults to "(Domino program dir)/jvm" |
| RustAddinLaunchWaitSecs       | Number of seconds to wait before starting the JVM; useful for some environments |
| RustAddinTempDirectory        | The directory to be used for "java.io.tmpdir" instead of the Java default |
| RustAddinJavaHeapInMB         | The size of the Java heap to override the default |
| RustAddinAdditionalJavaParams | A string of additional options to pass to the JVM |
| JavaUserOptionsFile           | Standard Domino INI parameter to specify the path to a file containing line-delimited options for the JVM |
| RustAddinJavaUserOptionsFile  | Keep-specific variant of `JavaUserOptionsFile` that overrides the above when present (including when present and blank) |

Each of these variables are loaded and processed in the ["main.rs" class in the Rust project](rust-addin/src/main.rs), so their mechanics can be observed there.