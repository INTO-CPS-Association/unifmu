# fmu identifier from model description
copy ..\modelDescription.xml container_bundle/modelDescription.xml
$str = Get-Content modelDescription.xml
$expr = 'guid="[^"]+'
$uid = [regex]::matches($str,$expr).value.substring(6)

# extract linux command 

# build image
docker build -t $uid .

# run container
docker run --net=host --rm $uid --entrypoint $args