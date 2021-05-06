function ok = instantiate()
global state_real;
global state_integer;
global state_boolean;
global state_string;
state_real = zeros(1, 3);
state_integer = zeros(1, 3, 'int32');
state_boolean = zeros(1, 3, 'logical');
state_string = strings(1, 3);
ok = int32(0);
