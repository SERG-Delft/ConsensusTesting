// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

/**
 * Protobuf enum {@code protocol.TMLedgerType}
 */
public enum TMLedgerType
    implements com.google.protobuf.ProtocolMessageEnum {
  /**
   * <code>ltACCEPTED = 0;</code>
   */
  ltACCEPTED(0),
  /**
   * <pre>
   * no longer supported
   * </pre>
   *
   * <code>ltCURRENT = 1;</code>
   */
  ltCURRENT(1),
  /**
   * <code>ltCLOSED = 2;</code>
   */
  ltCLOSED(2),
  ;

  /**
   * <code>ltACCEPTED = 0;</code>
   */
  public static final int ltACCEPTED_VALUE = 0;
  /**
   * <pre>
   * no longer supported
   * </pre>
   *
   * <code>ltCURRENT = 1;</code>
   */
  public static final int ltCURRENT_VALUE = 1;
  /**
   * <code>ltCLOSED = 2;</code>
   */
  public static final int ltCLOSED_VALUE = 2;


  public final int getNumber() {
    return value;
  }

  /**
   * @param value The numeric wire value of the corresponding enum entry.
   * @return The enum associated with the given numeric wire value.
   * @deprecated Use {@link #forNumber(int)} instead.
   */
  @java.lang.Deprecated
  public static TMLedgerType valueOf(int value) {
    return forNumber(value);
  }

  /**
   * @param value The numeric wire value of the corresponding enum entry.
   * @return The enum associated with the given numeric wire value.
   */
  public static TMLedgerType forNumber(int value) {
    switch (value) {
      case 0: return ltACCEPTED;
      case 1: return ltCURRENT;
      case 2: return ltCLOSED;
      default: return null;
    }
  }

  public static com.google.protobuf.Internal.EnumLiteMap<TMLedgerType>
      internalGetValueMap() {
    return internalValueMap;
  }
  private static final com.google.protobuf.Internal.EnumLiteMap<
      TMLedgerType> internalValueMap =
        new com.google.protobuf.Internal.EnumLiteMap<TMLedgerType>() {
          public TMLedgerType findValueByNumber(int number) {
            return TMLedgerType.forNumber(number);
          }
        };

  public final com.google.protobuf.Descriptors.EnumValueDescriptor
      getValueDescriptor() {
    return getDescriptor().getValues().get(ordinal());
  }
  public final com.google.protobuf.Descriptors.EnumDescriptor
      getDescriptorForType() {
    return getDescriptor();
  }
  public static final com.google.protobuf.Descriptors.EnumDescriptor
      getDescriptor() {
    return protocol.Ripple.getDescriptor().getEnumTypes().get(6);
  }

  private static final TMLedgerType[] VALUES = values();

  public static TMLedgerType valueOf(
      com.google.protobuf.Descriptors.EnumValueDescriptor desc) {
    if (desc.getType() != getDescriptor()) {
      throw new java.lang.IllegalArgumentException(
        "EnumValueDescriptor is not for this type.");
    }
    return VALUES[desc.getIndex()];
  }

  private final int value;

  private TMLedgerType(int value) {
    this.value = value;
  }

  // @@protoc_insertion_point(enum_scope:protocol.TMLedgerType)
}

