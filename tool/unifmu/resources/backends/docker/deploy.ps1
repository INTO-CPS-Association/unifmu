# fmu identifier from model description
$str = Get-Content ..\modelDescription.xml
copy ..\modelDescription.xml modelDescription.xml

$expr = 'guid="[^"]+'
$uid = [regex]::matches($str,$expr).value.substring(6)

# build image
docker build -t $uid .

# run container
docker run --net=host --rm $uid python bootstrap.py $args