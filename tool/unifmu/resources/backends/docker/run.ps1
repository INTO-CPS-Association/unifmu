# fmu identifier from model description
copy ..\modelDescription.xml modelDescription.xml
$str = Get-Content modelDescription.xml
$expr = 'guid="[^"]+'
$uid = [regex]::matches($str,$expr).value.substring(6)



# build image
docker build -t $uid .

# run container
docker run --net=host --rm $uid $args[0] $args[1]