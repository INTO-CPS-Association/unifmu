# UniFMU C# backend

This is unifmu's C# backend.

You do not need to compile the code as it is run directly by the `dotnet run` command. Just ensure that you have dotnet installed. 

The backend targets .NET 10 and requires the .NET 10 SDK to compile and run.

Run local backends with `dotnet run --project model.csproj --`. For a remote
backend, pass the proxy port after `--`, for example
`dotnet run --project model.csproj -- 5000`; without a port it prompts for one.

NetMQ 4.0.4.5 supplies updated .NET 10 dependencies, removing the vulnerable
System.Drawing.Common 5.0.0 and System.Security.Cryptography.Xml 5.0.0
transitive dependencies used by NetMQ 4.0.1.13. Earlier NetMQ 4.0.1.x updates
still introduce vulnerable cryptography dependencies.
