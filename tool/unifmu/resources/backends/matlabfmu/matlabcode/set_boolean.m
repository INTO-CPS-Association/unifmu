function ok = set_boolean(references, values)
global state_boolean;
n = numel(references);
for i = 1:n
    vr = references(i)-6+1;
    state_boolean(vr) = values(i);
end
ok = int32(0);
