# fmu identifier from model description
$str = Get-Content ..\modelDescription.xml
Copy-Item ..\modelDescription.xml container_bundle\modelDescription.xml

$expr = 'guid="[^"]+'
$uid = [regex]::matches($str, $expr).value.substring(6)

# build image
docker build -t $uid .

$max_tries = 10

for ($i = 0; $i -lt $max_tries; $i++) {
    $port = Get-Random -Max 32767 -Min 10001
    "unifmu: attempting to run container with port '$port' mapped from host to container."
    # docker run -d -p ${port}:${port} unifmu_a python3
    # check for ctrl+c (error code 137) https://betterprogramming.pub/understanding-docker-container-exit-codes-5ee79a1d58f6
    # https://github.com/INTO-CPS-Association/unifmu/issues/27
    docker run --rm -p ${port}:${port} $uid python bootstrap.py $args "--command-endpoint" "0.0.0.0:$port" "--use-docker-localhost"
    $code = $LASTEXITCODE

    if ($code -eq 0) {
        "unifmu: sucessfully bound to port '$port', container initialzing."
        exit 0
    }
    if ($code -eq 137) {
        "unifmu: the launch of docker was cancelled due to user invention. Aborting instantiation of docker container.".
        exit(-1)
    }
    else{
        "unifmu: failed binding to port '$port' after '$i' tries out of '$max_tries', retrying with another port."
    }

}
exit -1