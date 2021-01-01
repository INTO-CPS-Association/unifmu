// automatically generated by the FlatBuffers compiler, do not modify

package flatbuffers;

import java.nio.*;
import java.lang.*;
import java.util.*;
import com.google.flatbuffers.*;

@SuppressWarnings("unused")
public final class FMI2Command extends Table {
  public static void ValidateVersion() { Constants.FLATBUFFERS_1_12_0(); }
  public static FMI2Command getRootAsFMI2Command(ByteBuffer _bb) { return getRootAsFMI2Command(_bb, new FMI2Command()); }
  public static FMI2Command getRootAsFMI2Command(ByteBuffer _bb, FMI2Command obj) { _bb.order(ByteOrder.LITTLE_ENDIAN); return (obj.__assign(_bb.getInt(_bb.position()) + _bb.position(), _bb)); }
  public void __init(int _i, ByteBuffer _bb) { __reset(_i, _bb); }
  public FMI2Command __assign(int _i, ByteBuffer _bb) { __init(_i, _bb); return this; }

  public byte argsType() { int o = __offset(4); return o != 0 ? bb.get(o + bb_pos) : 0; }
  public Table args(Table obj) { int o = __offset(6); return o != 0 ? __union(obj, o + bb_pos) : null; }

  public static int createFMI2Command(FlatBufferBuilder builder,
      byte args_type,
      int argsOffset) {
    builder.startTable(2);
    FMI2Command.addArgs(builder, argsOffset);
    FMI2Command.addArgsType(builder, args_type);
    return FMI2Command.endFMI2Command(builder);
  }

  public static void startFMI2Command(FlatBufferBuilder builder) { builder.startTable(2); }
  public static void addArgsType(FlatBufferBuilder builder, byte argsType) { builder.addByte(0, argsType, 0); }
  public static void addArgs(FlatBufferBuilder builder, int argsOffset) { builder.addOffset(1, argsOffset, 0); }
  public static int endFMI2Command(FlatBufferBuilder builder) {
    int o = builder.endTable();
    return o;
  }
  public static void finishFMI2CommandBuffer(FlatBufferBuilder builder, int offset) { builder.finish(offset); }
  public static void finishSizePrefixedFMI2CommandBuffer(FlatBufferBuilder builder, int offset) { builder.finishSizePrefixed(offset); }

  public static final class Vector extends BaseVector {
    public Vector __assign(int _vector, int _element_size, ByteBuffer _bb) { __reset(_vector, _element_size, _bb); return this; }

    public FMI2Command get(int j) { return get(new FMI2Command(), j); }
    public FMI2Command get(FMI2Command obj, int j) {  return obj.__assign(__indirect(__element(j), bb), bb); }
  }
}

