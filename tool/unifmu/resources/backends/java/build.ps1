mvn clean compile assembly:single
java -jar .\target\unifmu-0.0.1-jar-with-dependencies.jar --handshake-endpoint "tcp://localhost:5000"