function ok = set_real(references, values)
global state_real;
n = numel(references);
for i = 1:n
    vr = references(i)+1;
    state_real(vr) = values(i);
end
ok = int32(0);
