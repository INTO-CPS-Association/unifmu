**This FMU was generated using UniFMU to work as a PROXY.
For general instructions on how to use the tool access the repository https://github.com/INTO-CPS-Association/unifmu**

# Using the Proxy
This FMU uses the proxy feature using the UniFMU API, behaving as any other FMU following the FMI standard, but interacting with its model counterpart on a different network client.

## Use of the ModelDescription.xml file
Therefore, each input, output or parameter declared in the `modelDescription.xml` file needs to be consistent with the attributes on the instance of the Model or FMU this proxy is featuring.

For instance, if a variable `a` is declared in the `modelDescription.xml` file, an attribute of the same name should be declared in the Model (of black-box FMU) running on the other network client:

```xml
<ScalarVariable name="a" valueReference="0" variability="continuous" causality="input">
    <Real start="0.0" />
</ScalarVariable>
```

## Executing the proxy and connecting to it from a model
To execute the proxy, run this FMU from a co-simulation master algorithm; it will log the information with the network port that is to be used by the model counterpart, as follows (this will be highlighted in color on the terminal):

```
...
Connect remote backend to dispatcher through port 42243
...
```

For instance, in this case, the model counterpart needs to connect to the IP address of the host running the co-simulation to the port 42243. **Notice that the port is allocated randomly and changes everytime.**

Once the proxy is waiting for a client, the model counterpart can initiate the connection via the backend of the `private` pair generated with UniFMU.