// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

/**
 * Protobuf enum {@code protocol.TransactionStatus}
 */
public enum TransactionStatus
    implements com.google.protobuf.ProtocolMessageEnum {
  /**
   * <pre>
   * origin node did/could not validate
   * </pre>
   *
   * <code>tsNEW = 1;</code>
   */
  tsNEW(1),
  /**
   * <pre>
   * scheduled to go in this ledger
   * </pre>
   *
   * <code>tsCURRENT = 2;</code>
   */
  tsCURRENT(2),
  /**
   * <pre>
   * in a closed ledger
   * </pre>
   *
   * <code>tsCOMMITED = 3;</code>
   */
  tsCOMMITED(3),
  /**
   * <code>tsREJECT_CONFLICT = 4;</code>
   */
  tsREJECT_CONFLICT(4),
  /**
   * <code>tsREJECT_INVALID = 5;</code>
   */
  tsREJECT_INVALID(5),
  /**
   * <code>tsREJECT_FUNDS = 6;</code>
   */
  tsREJECT_FUNDS(6),
  /**
   * <code>tsHELD_SEQ = 7;</code>
   */
  tsHELD_SEQ(7),
  /**
   * <pre>
   * held for future ledger
   * </pre>
   *
   * <code>tsHELD_LEDGER = 8;</code>
   */
  tsHELD_LEDGER(8),
  ;

  /**
   * <pre>
   * origin node did/could not validate
   * </pre>
   *
   * <code>tsNEW = 1;</code>
   */
  public static final int tsNEW_VALUE = 1;
  /**
   * <pre>
   * scheduled to go in this ledger
   * </pre>
   *
   * <code>tsCURRENT = 2;</code>
   */
  public static final int tsCURRENT_VALUE = 2;
  /**
   * <pre>
   * in a closed ledger
   * </pre>
   *
   * <code>tsCOMMITED = 3;</code>
   */
  public static final int tsCOMMITED_VALUE = 3;
  /**
   * <code>tsREJECT_CONFLICT = 4;</code>
   */
  public static final int tsREJECT_CONFLICT_VALUE = 4;
  /**
   * <code>tsREJECT_INVALID = 5;</code>
   */
  public static final int tsREJECT_INVALID_VALUE = 5;
  /**
   * <code>tsREJECT_FUNDS = 6;</code>
   */
  public static final int tsREJECT_FUNDS_VALUE = 6;
  /**
   * <code>tsHELD_SEQ = 7;</code>
   */
  public static final int tsHELD_SEQ_VALUE = 7;
  /**
   * <pre>
   * held for future ledger
   * </pre>
   *
   * <code>tsHELD_LEDGER = 8;</code>
   */
  public static final int tsHELD_LEDGER_VALUE = 8;


  public final int getNumber() {
    return value;
  }

  /**
   * @param value The numeric wire value of the corresponding enum entry.
   * @return The enum associated with the given numeric wire value.
   * @deprecated Use {@link #forNumber(int)} instead.
   */
  @java.lang.Deprecated
  public static TransactionStatus valueOf(int value) {
    return forNumber(value);
  }

  /**
   * @param value The numeric wire value of the corresponding enum entry.
   * @return The enum associated with the given numeric wire value.
   */
  public static TransactionStatus forNumber(int value) {
    switch (value) {
      case 1: return tsNEW;
      case 2: return tsCURRENT;
      case 3: return tsCOMMITED;
      case 4: return tsREJECT_CONFLICT;
      case 5: return tsREJECT_INVALID;
      case 6: return tsREJECT_FUNDS;
      case 7: return tsHELD_SEQ;
      case 8: return tsHELD_LEDGER;
      default: return null;
    }
  }

  public static com.google.protobuf.Internal.EnumLiteMap<TransactionStatus>
      internalGetValueMap() {
    return internalValueMap;
  }
  private static final com.google.protobuf.Internal.EnumLiteMap<
      TransactionStatus> internalValueMap =
        new com.google.protobuf.Internal.EnumLiteMap<TransactionStatus>() {
          public TransactionStatus findValueByNumber(int number) {
            return TransactionStatus.forNumber(number);
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
    return protocol.Ripple.getDescriptor().getEnumTypes().get(1);
  }

  private static final TransactionStatus[] VALUES = values();

  public static TransactionStatus valueOf(
      com.google.protobuf.Descriptors.EnumValueDescriptor desc) {
    if (desc.getType() != getDescriptor()) {
      throw new java.lang.IllegalArgumentException(
        "EnumValueDescriptor is not for this type.");
    }
    return VALUES[desc.getIndex()];
  }

  private final int value;

  private TransactionStatus(int value) {
    this.value = value;
  }

  // @@protoc_insertion_point(enum_scope:protocol.TransactionStatus)
}
