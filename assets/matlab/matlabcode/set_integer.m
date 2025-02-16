function ok = set_integer(references, values)
global state_integer;
n = numel(references);
for i = 1:n
    vr = references(i)-3+1;
    state_integer(vr) = values(i);
end
ok = int32(0);