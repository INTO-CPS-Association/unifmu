# fmu identifier from model description
$str = Get-Content modelDescription.xml
$expr = 'guid="[^"]+'
$uid = [regex]::matches($str,$expr).value.substring(6)

copy ..\modelDescription.xml modelDescription.xml

# build image
docker build -t $uid .

# run container
docker run --net=host --rm $uid $arg[0] $arg[1]