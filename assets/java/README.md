# UniFMU Java backend

This is unifmu's java backend.

You do not need to compile the code as this is done as part of FMU instantiation. Compiling the code will reduce the time it takes to instantiate the first FMU - succesive FMUs will use the build already cached.

If you want to compile the code before using the FMU, this is done using the "gradlew" scripts contained in the "resources" folder.

Depending on your operating system run the following command from within the "resources" folder to compile the binaries:

**windows**
    gradelw.bat compileJava --build-cache
**linux/mac**
    ./gradlew compileJava --build-cache

**Note:** Java version should be smaller or equal to java 17.
