// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

/**
 * Protobuf type {@code protocol.TMTransaction}
 */
public final class TMTransaction extends
    com.google.protobuf.GeneratedMessageV3 implements
    // @@protoc_insertion_point(message_implements:protocol.TMTransaction)
    TMTransactionOrBuilder {
private static final long serialVersionUID = 0L;
  // Use TMTransaction.newBuilder() to construct.
  private TMTransaction(com.google.protobuf.GeneratedMessageV3.Builder<?> builder) {
    super(builder);
  }
  private TMTransaction() {
    rawTransaction_ = com.google.protobuf.ByteString.EMPTY;
    status_ = 1;
  }

  @java.lang.Override
  @SuppressWarnings({"unused"})
  protected java.lang.Object newInstance(
      UnusedPrivateParameter unused) {
    return new TMTransaction();
  }

  @java.lang.Override
  public final com.google.protobuf.UnknownFieldSet
  getUnknownFields() {
    return this.unknownFields;
  }
  private TMTransaction(
      com.google.protobuf.CodedInputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    this();
    if (extensionRegistry == null) {
      throw new java.lang.NullPointerException();
    }
    int mutable_bitField0_ = 0;
    com.google.protobuf.UnknownFieldSet.Builder unknownFields =
        com.google.protobuf.UnknownFieldSet.newBuilder();
    try {
      boolean done = false;
      while (!done) {
        int tag = input.readTag();
        switch (tag) {
          case 0:
            done = true;
            break;
          case 10: {
            bitField0_ |= 0x00000001;
            rawTransaction_ = input.readBytes();
            break;
          }
          case 16: {
            int rawValue = input.readEnum();
              @SuppressWarnings("deprecation")
            protocol.TransactionStatus value = protocol.TransactionStatus.valueOf(rawValue);
            if (value == null) {
              unknownFields.mergeVarintField(2, rawValue);
            } else {
              bitField0_ |= 0x00000002;
              status_ = rawValue;
            }
            break;
          }
          case 24: {
            bitField0_ |= 0x00000004;
            receiveTimestamp_ = input.readUInt64();
            break;
          }
          case 32: {
            bitField0_ |= 0x00000008;
            deferred_ = input.readBool();
            break;
          }
          default: {
            if (!parseUnknownField(
                input, unknownFields, extensionRegistry, tag)) {
              done = true;
            }
            break;
          }
        }
      }
    } catch (com.google.protobuf.InvalidProtocolBufferException e) {
      throw e.setUnfinishedMessage(this);
    } catch (java.io.IOException e) {
      throw new com.google.protobuf.InvalidProtocolBufferException(
          e).setUnfinishedMessage(this);
    } finally {
      this.unknownFields = unknownFields.build();
      makeExtensionsImmutable();
    }
  }
  public static final com.google.protobuf.Descriptors.Descriptor
      getDescriptor() {
    return protocol.Ripple.internal_static_protocol_TMTransaction_descriptor;
  }

  @java.lang.Override
  protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internalGetFieldAccessorTable() {
    return protocol.Ripple.internal_static_protocol_TMTransaction_fieldAccessorTable
        .ensureFieldAccessorsInitialized(
            protocol.TMTransaction.class, protocol.TMTransaction.Builder.class);
  }

  private int bitField0_;
  public static final int RAWTRANSACTION_FIELD_NUMBER = 1;
  private com.google.protobuf.ByteString rawTransaction_;
  /**
   * <code>required bytes rawTransaction = 1;</code>
   * @return Whether the rawTransaction field is set.
   */
  @java.lang.Override
  public boolean hasRawTransaction() {
    return ((bitField0_ & 0x00000001) != 0);
  }
  /**
   * <code>required bytes rawTransaction = 1;</code>
   * @return The rawTransaction.
   */
  @java.lang.Override
  public com.google.protobuf.ByteString getRawTransaction() {
    return rawTransaction_;
  }

  public static final int STATUS_FIELD_NUMBER = 2;
  private int status_;
  /**
   * <code>required .protocol.TransactionStatus status = 2;</code>
   * @return Whether the status field is set.
   */
  @java.lang.Override public boolean hasStatus() {
    return ((bitField0_ & 0x00000002) != 0);
  }
  /**
   * <code>required .protocol.TransactionStatus status = 2;</code>
   * @return The status.
   */
  @java.lang.Override public protocol.TransactionStatus getStatus() {
    @SuppressWarnings("deprecation")
    protocol.TransactionStatus result = protocol.TransactionStatus.valueOf(status_);
    return result == null ? protocol.TransactionStatus.tsNEW : result;
  }

  public static final int RECEIVETIMESTAMP_FIELD_NUMBER = 3;
  private long receiveTimestamp_;
  /**
   * <code>optional uint64 receiveTimestamp = 3;</code>
   * @return Whether the receiveTimestamp field is set.
   */
  @java.lang.Override
  public boolean hasReceiveTimestamp() {
    return ((bitField0_ & 0x00000004) != 0);
  }
  /**
   * <code>optional uint64 receiveTimestamp = 3;</code>
   * @return The receiveTimestamp.
   */
  @java.lang.Override
  public long getReceiveTimestamp() {
    return receiveTimestamp_;
  }

  public static final int DEFERRED_FIELD_NUMBER = 4;
  private boolean deferred_;
  /**
   * <pre>
   * not applied to open ledger
   * </pre>
   *
   * <code>optional bool deferred = 4;</code>
   * @return Whether the deferred field is set.
   */
  @java.lang.Override
  public boolean hasDeferred() {
    return ((bitField0_ & 0x00000008) != 0);
  }
  /**
   * <pre>
   * not applied to open ledger
   * </pre>
   *
   * <code>optional bool deferred = 4;</code>
   * @return The deferred.
   */
  @java.lang.Override
  public boolean getDeferred() {
    return deferred_;
  }

  private byte memoizedIsInitialized = -1;
  @java.lang.Override
  public final boolean isInitialized() {
    byte isInitialized = memoizedIsInitialized;
    if (isInitialized == 1) return true;
    if (isInitialized == 0) return false;

    if (!hasRawTransaction()) {
      memoizedIsInitialized = 0;
      return false;
    }
    if (!hasStatus()) {
      memoizedIsInitialized = 0;
      return false;
    }
    memoizedIsInitialized = 1;
    return true;
  }

  @java.lang.Override
  public void writeTo(com.google.protobuf.CodedOutputStream output)
                      throws java.io.IOException {
    if (((bitField0_ & 0x00000001) != 0)) {
      output.writeBytes(1, rawTransaction_);
    }
    if (((bitField0_ & 0x00000002) != 0)) {
      output.writeEnum(2, status_);
    }
    if (((bitField0_ & 0x00000004) != 0)) {
      output.writeUInt64(3, receiveTimestamp_);
    }
    if (((bitField0_ & 0x00000008) != 0)) {
      output.writeBool(4, deferred_);
    }
    unknownFields.writeTo(output);
  }

  @java.lang.Override
  public int getSerializedSize() {
    int size = memoizedSize;
    if (size != -1) return size;

    size = 0;
    if (((bitField0_ & 0x00000001) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeBytesSize(1, rawTransaction_);
    }
    if (((bitField0_ & 0x00000002) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeEnumSize(2, status_);
    }
    if (((bitField0_ & 0x00000004) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeUInt64Size(3, receiveTimestamp_);
    }
    if (((bitField0_ & 0x00000008) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeBoolSize(4, deferred_);
    }
    size += unknownFields.getSerializedSize();
    memoizedSize = size;
    return size;
  }

  @java.lang.Override
  public boolean equals(final java.lang.Object obj) {
    if (obj == this) {
     return true;
    }
    if (!(obj instanceof protocol.TMTransaction)) {
      return super.equals(obj);
    }
    protocol.TMTransaction other = (protocol.TMTransaction) obj;

    if (hasRawTransaction() != other.hasRawTransaction()) return false;
    if (hasRawTransaction()) {
      if (!getRawTransaction()
          .equals(other.getRawTransaction())) return false;
    }
    if (hasStatus() != other.hasStatus()) return false;
    if (hasStatus()) {
      if (status_ != other.status_) return false;
    }
    if (hasReceiveTimestamp() != other.hasReceiveTimestamp()) return false;
    if (hasReceiveTimestamp()) {
      if (getReceiveTimestamp()
          != other.getReceiveTimestamp()) return false;
    }
    if (hasDeferred() != other.hasDeferred()) return false;
    if (hasDeferred()) {
      if (getDeferred()
          != other.getDeferred()) return false;
    }
    if (!unknownFields.equals(other.unknownFields)) return false;
    return true;
  }

  @java.lang.Override
  public int hashCode() {
    if (memoizedHashCode != 0) {
      return memoizedHashCode;
    }
    int hash = 41;
    hash = (19 * hash) + getDescriptor().hashCode();
    if (hasRawTransaction()) {
      hash = (37 * hash) + RAWTRANSACTION_FIELD_NUMBER;
      hash = (53 * hash) + getRawTransaction().hashCode();
    }
    if (hasStatus()) {
      hash = (37 * hash) + STATUS_FIELD_NUMBER;
      hash = (53 * hash) + status_;
    }
    if (hasReceiveTimestamp()) {
      hash = (37 * hash) + RECEIVETIMESTAMP_FIELD_NUMBER;
      hash = (53 * hash) + com.google.protobuf.Internal.hashLong(
          getReceiveTimestamp());
    }
    if (hasDeferred()) {
      hash = (37 * hash) + DEFERRED_FIELD_NUMBER;
      hash = (53 * hash) + com.google.protobuf.Internal.hashBoolean(
          getDeferred());
    }
    hash = (29 * hash) + unknownFields.hashCode();
    memoizedHashCode = hash;
    return hash;
  }

  public static protocol.TMTransaction parseFrom(
      java.nio.ByteBuffer data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMTransaction parseFrom(
      java.nio.ByteBuffer data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMTransaction parseFrom(
      com.google.protobuf.ByteString data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMTransaction parseFrom(
      com.google.protobuf.ByteString data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMTransaction parseFrom(byte[] data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMTransaction parseFrom(
      byte[] data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMTransaction parseFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMTransaction parseFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMTransaction parseDelimitedFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input);
  }
  public static protocol.TMTransaction parseDelimitedFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMTransaction parseFrom(
      com.google.protobuf.CodedInputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMTransaction parseFrom(
      com.google.protobuf.CodedInputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input, extensionRegistry);
  }

  @java.lang.Override
  public Builder newBuilderForType() { return newBuilder(); }
  public static Builder newBuilder() {
    return DEFAULT_INSTANCE.toBuilder();
  }
  public static Builder newBuilder(protocol.TMTransaction prototype) {
    return DEFAULT_INSTANCE.toBuilder().mergeFrom(prototype);
  }
  @java.lang.Override
  public Builder toBuilder() {
    return this == DEFAULT_INSTANCE
        ? new Builder() : new Builder().mergeFrom(this);
  }

  @java.lang.Override
  protected Builder newBuilderForType(
      com.google.protobuf.GeneratedMessageV3.BuilderParent parent) {
    Builder builder = new Builder(parent);
    return builder;
  }
  /**
   * Protobuf type {@code protocol.TMTransaction}
   */
  public static final class Builder extends
      com.google.protobuf.GeneratedMessageV3.Builder<Builder> implements
      // @@protoc_insertion_point(builder_implements:protocol.TMTransaction)
      protocol.TMTransactionOrBuilder {
    public static final com.google.protobuf.Descriptors.Descriptor
        getDescriptor() {
      return protocol.Ripple.internal_static_protocol_TMTransaction_descriptor;
    }

    @java.lang.Override
    protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
        internalGetFieldAccessorTable() {
      return protocol.Ripple.internal_static_protocol_TMTransaction_fieldAccessorTable
          .ensureFieldAccessorsInitialized(
              protocol.TMTransaction.class, protocol.TMTransaction.Builder.class);
    }

    // Construct using protocol.TMTransaction.newBuilder()
    private Builder() {
      maybeForceBuilderInitialization();
    }

    private Builder(
        com.google.protobuf.GeneratedMessageV3.BuilderParent parent) {
      super(parent);
      maybeForceBuilderInitialization();
    }
    private void maybeForceBuilderInitialization() {
      if (com.google.protobuf.GeneratedMessageV3
              .alwaysUseFieldBuilders) {
      }
    }
    @java.lang.Override
    public Builder clear() {
      super.clear();
      rawTransaction_ = com.google.protobuf.ByteString.EMPTY;
      bitField0_ = (bitField0_ & ~0x00000001);
      status_ = 1;
      bitField0_ = (bitField0_ & ~0x00000002);
      receiveTimestamp_ = 0L;
      bitField0_ = (bitField0_ & ~0x00000004);
      deferred_ = false;
      bitField0_ = (bitField0_ & ~0x00000008);
      return this;
    }

    @java.lang.Override
    public com.google.protobuf.Descriptors.Descriptor
        getDescriptorForType() {
      return protocol.Ripple.internal_static_protocol_TMTransaction_descriptor;
    }

    @java.lang.Override
    public protocol.TMTransaction getDefaultInstanceForType() {
      return protocol.TMTransaction.getDefaultInstance();
    }

    @java.lang.Override
    public protocol.TMTransaction build() {
      protocol.TMTransaction result = buildPartial();
      if (!result.isInitialized()) {
        throw newUninitializedMessageException(result);
      }
      return result;
    }

    @java.lang.Override
    public protocol.TMTransaction buildPartial() {
      protocol.TMTransaction result = new protocol.TMTransaction(this);
      int from_bitField0_ = bitField0_;
      int to_bitField0_ = 0;
      if (((from_bitField0_ & 0x00000001) != 0)) {
        to_bitField0_ |= 0x00000001;
      }
      result.rawTransaction_ = rawTransaction_;
      if (((from_bitField0_ & 0x00000002) != 0)) {
        to_bitField0_ |= 0x00000002;
      }
      result.status_ = status_;
      if (((from_bitField0_ & 0x00000004) != 0)) {
        result.receiveTimestamp_ = receiveTimestamp_;
        to_bitField0_ |= 0x00000004;
      }
      if (((from_bitField0_ & 0x00000008) != 0)) {
        result.deferred_ = deferred_;
        to_bitField0_ |= 0x00000008;
      }
      result.bitField0_ = to_bitField0_;
      onBuilt();
      return result;
    }

    @java.lang.Override
    public Builder clone() {
      return super.clone();
    }
    @java.lang.Override
    public Builder setField(
        com.google.protobuf.Descriptors.FieldDescriptor field,
        java.lang.Object value) {
      return super.setField(field, value);
    }
    @java.lang.Override
    public Builder clearField(
        com.google.protobuf.Descriptors.FieldDescriptor field) {
      return super.clearField(field);
    }
    @java.lang.Override
    public Builder clearOneof(
        com.google.protobuf.Descriptors.OneofDescriptor oneof) {
      return super.clearOneof(oneof);
    }
    @java.lang.Override
    public Builder setRepeatedField(
        com.google.protobuf.Descriptors.FieldDescriptor field,
        int index, java.lang.Object value) {
      return super.setRepeatedField(field, index, value);
    }
    @java.lang.Override
    public Builder addRepeatedField(
        com.google.protobuf.Descriptors.FieldDescriptor field,
        java.lang.Object value) {
      return super.addRepeatedField(field, value);
    }
    @java.lang.Override
    public Builder mergeFrom(com.google.protobuf.Message other) {
      if (other instanceof protocol.TMTransaction) {
        return mergeFrom((protocol.TMTransaction)other);
      } else {
        super.mergeFrom(other);
        return this;
      }
    }

    public Builder mergeFrom(protocol.TMTransaction other) {
      if (other == protocol.TMTransaction.getDefaultInstance()) return this;
      if (other.hasRawTransaction()) {
        setRawTransaction(other.getRawTransaction());
      }
      if (other.hasStatus()) {
        setStatus(other.getStatus());
      }
      if (other.hasReceiveTimestamp()) {
        setReceiveTimestamp(other.getReceiveTimestamp());
      }
      if (other.hasDeferred()) {
        setDeferred(other.getDeferred());
      }
      this.mergeUnknownFields(other.unknownFields);
      onChanged();
      return this;
    }

    @java.lang.Override
    public final boolean isInitialized() {
      if (!hasRawTransaction()) {
        return false;
      }
      if (!hasStatus()) {
        return false;
      }
      return true;
    }

    @java.lang.Override
    public Builder mergeFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws java.io.IOException {
      protocol.TMTransaction parsedMessage = null;
      try {
        parsedMessage = PARSER.parsePartialFrom(input, extensionRegistry);
      } catch (com.google.protobuf.InvalidProtocolBufferException e) {
        parsedMessage = (protocol.TMTransaction) e.getUnfinishedMessage();
        throw e.unwrapIOException();
      } finally {
        if (parsedMessage != null) {
          mergeFrom(parsedMessage);
        }
      }
      return this;
    }
    private int bitField0_;

    private com.google.protobuf.ByteString rawTransaction_ = com.google.protobuf.ByteString.EMPTY;
    /**
     * <code>required bytes rawTransaction = 1;</code>
     * @return Whether the rawTransaction field is set.
     */
    @java.lang.Override
    public boolean hasRawTransaction() {
      return ((bitField0_ & 0x00000001) != 0);
    }
    /**
     * <code>required bytes rawTransaction = 1;</code>
     * @return The rawTransaction.
     */
    @java.lang.Override
    public com.google.protobuf.ByteString getRawTransaction() {
      return rawTransaction_;
    }
    /**
     * <code>required bytes rawTransaction = 1;</code>
     * @param value The rawTransaction to set.
     * @return This builder for chaining.
     */
    public Builder setRawTransaction(com.google.protobuf.ByteString value) {
      if (value == null) {
    throw new NullPointerException();
  }
  bitField0_ |= 0x00000001;
      rawTransaction_ = value;
      onChanged();
      return this;
    }
    /**
     * <code>required bytes rawTransaction = 1;</code>
     * @return This builder for chaining.
     */
    public Builder clearRawTransaction() {
      bitField0_ = (bitField0_ & ~0x00000001);
      rawTransaction_ = getDefaultInstance().getRawTransaction();
      onChanged();
      return this;
    }

    private int status_ = 1;
    /**
     * <code>required .protocol.TransactionStatus status = 2;</code>
     * @return Whether the status field is set.
     */
    @java.lang.Override public boolean hasStatus() {
      return ((bitField0_ & 0x00000002) != 0);
    }
    /**
     * <code>required .protocol.TransactionStatus status = 2;</code>
     * @return The status.
     */
    @java.lang.Override
    public protocol.TransactionStatus getStatus() {
      @SuppressWarnings("deprecation")
      protocol.TransactionStatus result = protocol.TransactionStatus.valueOf(status_);
      return result == null ? protocol.TransactionStatus.tsNEW : result;
    }
    /**
     * <code>required .protocol.TransactionStatus status = 2;</code>
     * @param value The status to set.
     * @return This builder for chaining.
     */
    public Builder setStatus(protocol.TransactionStatus value) {
      if (value == null) {
        throw new NullPointerException();
      }
      bitField0_ |= 0x00000002;
      status_ = value.getNumber();
      onChanged();
      return this;
    }
    /**
     * <code>required .protocol.TransactionStatus status = 2;</code>
     * @return This builder for chaining.
     */
    public Builder clearStatus() {
      bitField0_ = (bitField0_ & ~0x00000002);
      status_ = 1;
      onChanged();
      return this;
    }

    private long receiveTimestamp_ ;
    /**
     * <code>optional uint64 receiveTimestamp = 3;</code>
     * @return Whether the receiveTimestamp field is set.
     */
    @java.lang.Override
    public boolean hasReceiveTimestamp() {
      return ((bitField0_ & 0x00000004) != 0);
    }
    /**
     * <code>optional uint64 receiveTimestamp = 3;</code>
     * @return The receiveTimestamp.
     */
    @java.lang.Override
    public long getReceiveTimestamp() {
      return receiveTimestamp_;
    }
    /**
     * <code>optional uint64 receiveTimestamp = 3;</code>
     * @param value The receiveTimestamp to set.
     * @return This builder for chaining.
     */
    public Builder setReceiveTimestamp(long value) {
      bitField0_ |= 0x00000004;
      receiveTimestamp_ = value;
      onChanged();
      return this;
    }
    /**
     * <code>optional uint64 receiveTimestamp = 3;</code>
     * @return This builder for chaining.
     */
    public Builder clearReceiveTimestamp() {
      bitField0_ = (bitField0_ & ~0x00000004);
      receiveTimestamp_ = 0L;
      onChanged();
      return this;
    }

    private boolean deferred_ ;
    /**
     * <pre>
     * not applied to open ledger
     * </pre>
     *
     * <code>optional bool deferred = 4;</code>
     * @return Whether the deferred field is set.
     */
    @java.lang.Override
    public boolean hasDeferred() {
      return ((bitField0_ & 0x00000008) != 0);
    }
    /**
     * <pre>
     * not applied to open ledger
     * </pre>
     *
     * <code>optional bool deferred = 4;</code>
     * @return The deferred.
     */
    @java.lang.Override
    public boolean getDeferred() {
      return deferred_;
    }
    /**
     * <pre>
     * not applied to open ledger
     * </pre>
     *
     * <code>optional bool deferred = 4;</code>
     * @param value The deferred to set.
     * @return This builder for chaining.
     */
    public Builder setDeferred(boolean value) {
      bitField0_ |= 0x00000008;
      deferred_ = value;
      onChanged();
      return this;
    }
    /**
     * <pre>
     * not applied to open ledger
     * </pre>
     *
     * <code>optional bool deferred = 4;</code>
     * @return This builder for chaining.
     */
    public Builder clearDeferred() {
      bitField0_ = (bitField0_ & ~0x00000008);
      deferred_ = false;
      onChanged();
      return this;
    }
    @java.lang.Override
    public final Builder setUnknownFields(
        final com.google.protobuf.UnknownFieldSet unknownFields) {
      return super.setUnknownFields(unknownFields);
    }

    @java.lang.Override
    public final Builder mergeUnknownFields(
        final com.google.protobuf.UnknownFieldSet unknownFields) {
      return super.mergeUnknownFields(unknownFields);
    }


    // @@protoc_insertion_point(builder_scope:protocol.TMTransaction)
  }

  // @@protoc_insertion_point(class_scope:protocol.TMTransaction)
  private static final protocol.TMTransaction DEFAULT_INSTANCE;
  static {
    DEFAULT_INSTANCE = new protocol.TMTransaction();
  }

  public static protocol.TMTransaction getDefaultInstance() {
    return DEFAULT_INSTANCE;
  }

  @java.lang.Deprecated public static final com.google.protobuf.Parser<TMTransaction>
      PARSER = new com.google.protobuf.AbstractParser<TMTransaction>() {
    @java.lang.Override
    public TMTransaction parsePartialFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws com.google.protobuf.InvalidProtocolBufferException {
      return new TMTransaction(input, extensionRegistry);
    }
  };

  public static com.google.protobuf.Parser<TMTransaction> parser() {
    return PARSER;
  }

  @java.lang.Override
  public com.google.protobuf.Parser<TMTransaction> getParserForType() {
    return PARSER;
  }

  @java.lang.Override
  public protocol.TMTransaction getDefaultInstanceForType() {
    return DEFAULT_INSTANCE;
  }

}

