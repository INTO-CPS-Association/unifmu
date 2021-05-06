function ok = deserialize(bytes)
global state_real;
global state_integer;
global state_boolean;
global state_string;
state = getArrayFromByteStream(bytes);
state_real = state{1};
state_integer = state{2};
state_boolean = state{3};
state_string = state{4};
ok = int32(0);
