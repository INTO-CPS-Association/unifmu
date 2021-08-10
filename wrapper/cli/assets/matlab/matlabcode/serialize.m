function [ok, bytes] = serialize()
global state_real;
global state_integer;
global state_boolean;
global state_string;
bytes = getByteStreamFromArray({state_real, state_integer, state_boolean, state_string});
ok = int32(0);
