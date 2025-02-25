function ok = set_string(references, values)
global state_string;
n = numel(references);
for i = 1:n
    vr = references(i)-9+1;
    state_string(vr) = values(i);
end
ok = int32(0);