# Domino Rust JVM Addin Example Example

This repository contains an example of writing a native Domino addin in Rust that loads a JVM and hands control over to a Java class.

This is similar in concept to [the RunJava task](https://paulswithers.github.io/blog/2020/03/01/runjava), but differs in some important ways:

- The JARs that make up the addin do not have to be placed in the global classpath (e.g. in ndext)
- The use of a native addin component allows for fine-grained control of the JVM, such as passing additional options, adjusting the heap size, or using a different JVM
- Since the example class here uses JNX, it is not required to use `lotus.domino.*` classes (though those would also work, including making a class that extends `lotus.notes.addins.JavaServerAddin` as the entrypoint)
- The Rust class could also be used to be the addin itself, or to spawn a different environment - the use of Java is a nice example, but not required for the native portion

## Preparation

Before building, copy the "libnotes.so", "libxmlproc.so", and "libndgts.so" files from the program directory of a Linux Domino installation to the "rust-addin/lib" directory. These files are not redistributable and so are not included in this repository.

## Building

### Building With Docker

The quickest way to build this project is to have a local Docker-compatible environment (such as Docker Desktop) and to run:

```
docker-compose up
```

That will run the builder container, which will compile the native library for Windows and Linux, package the Java module, and build the distribution.

Once built, the distribution ZIP with instructions will be "addin-dist/target/Rust-Addin-Example-1.0.0-SNAPSHOT.zip".

### Building Locally Without Docker

Alternatively, the project and its native component can be built on macOS or Linux (but not Windows) with a local Rust installation. To install Rust and the cross-compilation components, follow the first part of the instructions for your platform in the [native addin README](rust-addin/README.md).

Additionally, install Maven either from your system's package manager (such as [Homebrew](https://brew.sh/) on macOS) or from [maven.apache.org](https://maven.apache.org/).

Once these are installed, run:

```
mvn clean install -Pbuild-local,\!build-docker
```

This will tell the native module to use the local environment and not try to build using a Docker container.