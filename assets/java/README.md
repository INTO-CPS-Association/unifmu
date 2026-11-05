# UniFMU Java backend

Generated Java FMUs contain a small Gradle application project, a versioned
`libs/unifmu-java-runtime-<version>.jar`, and one editable example model source.
The runtime implementation is compiled into the JAR rather than copied as Java
source into each FMU. The generated `launch.toml` selects the included model and
the FMI version.

The editable model is `src/main/java/org/intocps/unifmu/example/fmi2/Model.java`
or `.../example/fmi3/Model.java`. Change this class to implement your FMU model;
the same example class is included in the runtime JAR for tests and standalone
library users. The FMU's source class takes precedence over that example class
when the generated project is compiled.

Runtime APIs are organized under `org.intocps.unifmu`: communication
interfaces and ZeroMQ transport are in `communication`, FMI model contracts and
factories are in `model.fmi2` / `model.fmi3`, protobuf bindings are in
`protocol`, and command handlers are in `protocol.fmi2` / `protocol.fmi3`.
`org.intocps.unifmu.Main` is the shared launcher in the runtime JAR.

## Selecting another model

The generated `launch.toml` passes `unifmu.model.impl` to the Gradle `run` task.
To use a different class, update that property in `launch.toml` or invoke Gradle
with `-Dunifmu.model.impl=com.example.CustomModel`. The selected class must
implement `Fmi2Model` or `Fmi3Model` and provide a public constructor accepting
the corresponding generated instantiate message and a `Consumer` log callback:

```java
public CustomModel(
    Fmi3Messages.Fmi3InstantiateCoSimulation instantiation,
    Consumer<Fmi3Model.LogMessage> logHandler
) { ... }
```

Set `unifmu.fmi.version` to `fmi2` or `fmi3` and `unifmu.launch.mode` to `local`
or `remote` when launching the runtime. The generated `launch.toml` sets all
three properties; for a custom project, update its launch command, for example:

```bash
./gradlew -Dunifmu.launch.mode=local -Dunifmu.fmi.version=fmi3 \
  -Dunifmu.model.impl=com.example.CustomModel run
```

Missing, incompatible, or improperly constructed model classes fail with a
descriptive error. `unifmu.channel.impl` can select an alternate
`CommunicationChannel` implementation with a public `String` endpoint
constructor; test-only fixtures are kept in `cli/tests/java_fixtures`.

## Building and using the runtime JAR

Build the versioned, self-contained runtime JAR with:

```bash
./assets/java/gradlew -p assets/java/runtime shadowJar
```

Generated FMUs receive this JAR under their `resources/libs` directory. To
install the same artifact into the local Maven repository:

```bash
./assets/java/gradlew -p assets/java/runtime publishToMavenLocal
```

Then add it to another Maven project (substitute the current UniFMU version):

```xml
<dependency>
  <groupId>org.intocps.unifmu</groupId>
  <artifactId>unifmu-java-runtime</artifactId>
  <version>0.14.0</version>
</dependency>
```

The artifact includes FMI2/FMI3 contracts, protocol handlers, protobuf messages,
transport support, the shared launcher, and example models. Its version comes
from the root `Cargo.toml`. Publishing it to a shared Maven repository requires
that repository's credentials and configuration; `publishToMavenLocal` is for
local development.

The Java runtime requires Java 21 or newer. Java sources in generated FMUs are
compiled on first backend startup. To compile ahead of time, run the Gradle
wrapper from the generated FMU's `resources` directory:

```bash
./gradlew compileJava --build-cache
```
