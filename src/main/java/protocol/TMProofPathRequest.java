// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

/**
 * Protobuf type {@code protocol.TMProofPathRequest}
 */
public final class TMProofPathRequest extends
    com.google.protobuf.GeneratedMessageV3 implements
    // @@protoc_insertion_point(message_implements:protocol.TMProofPathRequest)
    TMProofPathRequestOrBuilder {
private static final long serialVersionUID = 0L;
  // Use TMProofPathRequest.newBuilder() to construct.
  private TMProofPathRequest(com.google.protobuf.GeneratedMessageV3.Builder<?> builder) {
    super(builder);
  }
  private TMProofPathRequest() {
    key_ = com.google.protobuf.ByteString.EMPTY;
    ledgerHash_ = com.google.protobuf.ByteString.EMPTY;
    type_ = 1;
  }

  @java.lang.Override
  @SuppressWarnings({"unused"})
  protected java.lang.Object newInstance(
      UnusedPrivateParameter unused) {
    return new TMProofPathRequest();
  }

  @java.lang.Override
  public final com.google.protobuf.UnknownFieldSet
  getUnknownFields() {
    return this.unknownFields;
  }
  private TMProofPathRequest(
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
            key_ = input.readBytes();
            break;
          }
          case 18: {
            bitField0_ |= 0x00000002;
            ledgerHash_ = input.readBytes();
            break;
          }
          case 24: {
            int rawValue = input.readEnum();
              @SuppressWarnings("deprecation")
            protocol.TMLedgerMapType value = protocol.TMLedgerMapType.valueOf(rawValue);
            if (value == null) {
              unknownFields.mergeVarintField(3, rawValue);
            } else {
              bitField0_ |= 0x00000004;
              type_ = rawValue;
            }
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
    return protocol.Ripple.internal_static_protocol_TMProofPathRequest_descriptor;
  }

  @java.lang.Override
  protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internalGetFieldAccessorTable() {
    return protocol.Ripple.internal_static_protocol_TMProofPathRequest_fieldAccessorTable
        .ensureFieldAccessorsInitialized(
            protocol.TMProofPathRequest.class, protocol.TMProofPathRequest.Builder.class);
  }

  private int bitField0_;
  public static final int KEY_FIELD_NUMBER = 1;
  private com.google.protobuf.ByteString key_;
  /**
   * <code>required bytes key = 1;</code>
   * @return Whether the key field is set.
   */
  @java.lang.Override
  public boolean hasKey() {
    return ((bitField0_ & 0x00000001) != 0);
  }
  /**
   * <code>required bytes key = 1;</code>
   * @return The key.
   */
  @java.lang.Override
  public com.google.protobuf.ByteString getKey() {
    return key_;
  }

  public static final int LEDGERHASH_FIELD_NUMBER = 2;
  private com.google.protobuf.ByteString ledgerHash_;
  /**
   * <code>required bytes ledgerHash = 2;</code>
   * @return Whether the ledgerHash field is set.
   */
  @java.lang.Override
  public boolean hasLedgerHash() {
    return ((bitField0_ & 0x00000002) != 0);
  }
  /**
   * <code>required bytes ledgerHash = 2;</code>
   * @return The ledgerHash.
   */
  @java.lang.Override
  public com.google.protobuf.ByteString getLedgerHash() {
    return ledgerHash_;
  }

  public static final int TYPE_FIELD_NUMBER = 3;
  private int type_;
  /**
   * <code>required .protocol.TMLedgerMapType type = 3;</code>
   * @return Whether the type field is set.
   */
  @java.lang.Override public boolean hasType() {
    return ((bitField0_ & 0x00000004) != 0);
  }
  /**
   * <code>required .protocol.TMLedgerMapType type = 3;</code>
   * @return The type.
   */
  @java.lang.Override public protocol.TMLedgerMapType getType() {
    @SuppressWarnings("deprecation")
    protocol.TMLedgerMapType result = protocol.TMLedgerMapType.valueOf(type_);
    return result == null ? protocol.TMLedgerMapType.lmTRANASCTION : result;
  }

  private byte memoizedIsInitialized = -1;
  @java.lang.Override
  public final boolean isInitialized() {
    byte isInitialized = memoizedIsInitialized;
    if (isInitialized == 1) return true;
    if (isInitialized == 0) return false;

    if (!hasKey()) {
      memoizedIsInitialized = 0;
      return false;
    }
    if (!hasLedgerHash()) {
      memoizedIsInitialized = 0;
      return false;
    }
    if (!hasType()) {
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
      output.writeBytes(1, key_);
    }
    if (((bitField0_ & 0x00000002) != 0)) {
      output.writeBytes(2, ledgerHash_);
    }
    if (((bitField0_ & 0x00000004) != 0)) {
      output.writeEnum(3, type_);
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
        .computeBytesSize(1, key_);
    }
    if (((bitField0_ & 0x00000002) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeBytesSize(2, ledgerHash_);
    }
    if (((bitField0_ & 0x00000004) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeEnumSize(3, type_);
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
    if (!(obj instanceof protocol.TMProofPathRequest)) {
      return super.equals(obj);
    }
    protocol.TMProofPathRequest other = (protocol.TMProofPathRequest) obj;

    if (hasKey() != other.hasKey()) return false;
    if (hasKey()) {
      if (!getKey()
          .equals(other.getKey())) return false;
    }
    if (hasLedgerHash() != other.hasLedgerHash()) return false;
    if (hasLedgerHash()) {
      if (!getLedgerHash()
          .equals(other.getLedgerHash())) return false;
    }
    if (hasType() != other.hasType()) return false;
    if (hasType()) {
      if (type_ != other.type_) return false;
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
    if (hasKey()) {
      hash = (37 * hash) + KEY_FIELD_NUMBER;
      hash = (53 * hash) + getKey().hashCode();
    }
    if (hasLedgerHash()) {
      hash = (37 * hash) + LEDGERHASH_FIELD_NUMBER;
      hash = (53 * hash) + getLedgerHash().hashCode();
    }
    if (hasType()) {
      hash = (37 * hash) + TYPE_FIELD_NUMBER;
      hash = (53 * hash) + type_;
    }
    hash = (29 * hash) + unknownFields.hashCode();
    memoizedHashCode = hash;
    return hash;
  }

  public static protocol.TMProofPathRequest parseFrom(
      java.nio.ByteBuffer data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMProofPathRequest parseFrom(
      java.nio.ByteBuffer data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMProofPathRequest parseFrom(
      com.google.protobuf.ByteString data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMProofPathRequest parseFrom(
      com.google.protobuf.ByteString data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMProofPathRequest parseFrom(byte[] data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMProofPathRequest parseFrom(
      byte[] data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMProofPathRequest parseFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMProofPathRequest parseFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMProofPathRequest parseDelimitedFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input);
  }
  public static protocol.TMProofPathRequest parseDelimitedFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMProofPathRequest parseFrom(
      com.google.protobuf.CodedInputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMProofPathRequest parseFrom(
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
  public static Builder newBuilder(protocol.TMProofPathRequest prototype) {
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
   * Protobuf type {@code protocol.TMProofPathRequest}
   */
  public static final class Builder extends
      com.google.protobuf.GeneratedMessageV3.Builder<Builder> implements
      // @@protoc_insertion_point(builder_implements:protocol.TMProofPathRequest)
      protocol.TMProofPathRequestOrBuilder {
    public static final com.google.protobuf.Descriptors.Descriptor
        getDescriptor() {
      return protocol.Ripple.internal_static_protocol_TMProofPathRequest_descriptor;
    }

    @java.lang.Override
    protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
        internalGetFieldAccessorTable() {
      return protocol.Ripple.internal_static_protocol_TMProofPathRequest_fieldAccessorTable
          .ensureFieldAccessorsInitialized(
              protocol.TMProofPathRequest.class, protocol.TMProofPathRequest.Builder.class);
    }

    // Construct using protocol.TMProofPathRequest.newBuilder()
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
      key_ = com.google.protobuf.ByteString.EMPTY;
      bitField0_ = (bitField0_ & ~0x00000001);
      ledgerHash_ = com.google.protobuf.ByteString.EMPTY;
      bitField0_ = (bitField0_ & ~0x00000002);
      type_ = 1;
      bitField0_ = (bitField0_ & ~0x00000004);
      return this;
    }

    @java.lang.Override
    public com.google.protobuf.Descriptors.Descriptor
        getDescriptorForType() {
      return protocol.Ripple.internal_static_protocol_TMProofPathRequest_descriptor;
    }

    @java.lang.Override
    public protocol.TMProofPathRequest getDefaultInstanceForType() {
      return protocol.TMProofPathRequest.getDefaultInstance();
    }

    @java.lang.Override
    public protocol.TMProofPathRequest build() {
      protocol.TMProofPathRequest result = buildPartial();
      if (!result.isInitialized()) {
        throw newUninitializedMessageException(result);
      }
      return result;
    }

    @java.lang.Override
    public protocol.TMProofPathRequest buildPartial() {
      protocol.TMProofPathRequest result = new protocol.TMProofPathRequest(this);
      int from_bitField0_ = bitField0_;
      int to_bitField0_ = 0;
      if (((from_bitField0_ & 0x00000001) != 0)) {
        to_bitField0_ |= 0x00000001;
      }
      result.key_ = key_;
      if (((from_bitField0_ & 0x00000002) != 0)) {
        to_bitField0_ |= 0x00000002;
      }
      result.ledgerHash_ = ledgerHash_;
      if (((from_bitField0_ & 0x00000004) != 0)) {
        to_bitField0_ |= 0x00000004;
      }
      result.type_ = type_;
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
      if (other instanceof protocol.TMProofPathRequest) {
        return mergeFrom((protocol.TMProofPathRequest)other);
      } else {
        super.mergeFrom(other);
        return this;
      }
    }

    public Builder mergeFrom(protocol.TMProofPathRequest other) {
      if (other == protocol.TMProofPathRequest.getDefaultInstance()) return this;
      if (other.hasKey()) {
        setKey(other.getKey());
      }
      if (other.hasLedgerHash()) {
        setLedgerHash(other.getLedgerHash());
      }
      if (other.hasType()) {
        setType(other.getType());
      }
      this.mergeUnknownFields(other.unknownFields);
      onChanged();
      return this;
    }

    @java.lang.Override
    public final boolean isInitialized() {
      if (!hasKey()) {
        return false;
      }
      if (!hasLedgerHash()) {
        return false;
      }
      if (!hasType()) {
        return false;
      }
      return true;
    }

    @java.lang.Override
    public Builder mergeFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws java.io.IOException {
      protocol.TMProofPathRequest parsedMessage = null;
      try {
        parsedMessage = PARSER.parsePartialFrom(input, extensionRegistry);
      } catch (com.google.protobuf.InvalidProtocolBufferException e) {
        parsedMessage = (protocol.TMProofPathRequest) e.getUnfinishedMessage();
        throw e.unwrapIOException();
      } finally {
        if (parsedMessage != null) {
          mergeFrom(parsedMessage);
        }
      }
      return this;
    }
    private int bitField0_;

    private com.google.protobuf.ByteString key_ = com.google.protobuf.ByteString.EMPTY;
    /**
     * <code>required bytes key = 1;</code>
     * @return Whether the key field is set.
     */
    @java.lang.Override
    public boolean hasKey() {
      return ((bitField0_ & 0x00000001) != 0);
    }
    /**
     * <code>required bytes key = 1;</code>
     * @return The key.
     */
    @java.lang.Override
    public com.google.protobuf.ByteString getKey() {
      return key_;
    }
    /**
     * <code>required bytes key = 1;</code>
     * @param value The key to set.
     * @return This builder for chaining.
     */
    public Builder setKey(com.google.protobuf.ByteString value) {
      if (value == null) {
    throw new NullPointerException();
  }
  bitField0_ |= 0x00000001;
      key_ = value;
      onChanged();
      return this;
    }
    /**
     * <code>required bytes key = 1;</code>
     * @return This builder for chaining.
     */
    public Builder clearKey() {
      bitField0_ = (bitField0_ & ~0x00000001);
      key_ = getDefaultInstance().getKey();
      onChanged();
      return this;
    }

    private com.google.protobuf.ByteString ledgerHash_ = com.google.protobuf.ByteString.EMPTY;
    /**
     * <code>required bytes ledgerHash = 2;</code>
     * @return Whether the ledgerHash field is set.
     */
    @java.lang.Override
    public boolean hasLedgerHash() {
      return ((bitField0_ & 0x00000002) != 0);
    }
    /**
     * <code>required bytes ledgerHash = 2;</code>
     * @return The ledgerHash.
     */
    @java.lang.Override
    public com.google.protobuf.ByteString getLedgerHash() {
      return ledgerHash_;
    }
    /**
     * <code>required bytes ledgerHash = 2;</code>
     * @param value The ledgerHash to set.
     * @return This builder for chaining.
     */
    public Builder setLedgerHash(com.google.protobuf.ByteString value) {
      if (value == null) {
    throw new NullPointerException();
  }
  bitField0_ |= 0x00000002;
      ledgerHash_ = value;
      onChanged();
      return this;
    }
    /**
     * <code>required bytes ledgerHash = 2;</code>
     * @return This builder for chaining.
     */
    public Builder clearLedgerHash() {
      bitField0_ = (bitField0_ & ~0x00000002);
      ledgerHash_ = getDefaultInstance().getLedgerHash();
      onChanged();
      return this;
    }

    private int type_ = 1;
    /**
     * <code>required .protocol.TMLedgerMapType type = 3;</code>
     * @return Whether the type field is set.
     */
    @java.lang.Override public boolean hasType() {
      return ((bitField0_ & 0x00000004) != 0);
    }
    /**
     * <code>required .protocol.TMLedgerMapType type = 3;</code>
     * @return The type.
     */
    @java.lang.Override
    public protocol.TMLedgerMapType getType() {
      @SuppressWarnings("deprecation")
      protocol.TMLedgerMapType result = protocol.TMLedgerMapType.valueOf(type_);
      return result == null ? protocol.TMLedgerMapType.lmTRANASCTION : result;
    }
    /**
     * <code>required .protocol.TMLedgerMapType type = 3;</code>
     * @param value The type to set.
     * @return This builder for chaining.
     */
    public Builder setType(protocol.TMLedgerMapType value) {
      if (value == null) {
        throw new NullPointerException();
      }
      bitField0_ |= 0x00000004;
      type_ = value.getNumber();
      onChanged();
      return this;
    }
    /**
     * <code>required .protocol.TMLedgerMapType type = 3;</code>
     * @return This builder for chaining.
     */
    public Builder clearType() {
      bitField0_ = (bitField0_ & ~0x00000004);
      type_ = 1;
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


    // @@protoc_insertion_point(builder_scope:protocol.TMProofPathRequest)
  }

  // @@protoc_insertion_point(class_scope:protocol.TMProofPathRequest)
  private static final protocol.TMProofPathRequest DEFAULT_INSTANCE;
  static {
    DEFAULT_INSTANCE = new protocol.TMProofPathRequest();
  }

  public static protocol.TMProofPathRequest getDefaultInstance() {
    return DEFAULT_INSTANCE;
  }

  @java.lang.Deprecated public static final com.google.protobuf.Parser<TMProofPathRequest>
      PARSER = new com.google.protobuf.AbstractParser<TMProofPathRequest>() {
    @java.lang.Override
    public TMProofPathRequest parsePartialFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws com.google.protobuf.InvalidProtocolBufferException {
      return new TMProofPathRequest(input, extensionRegistry);
    }
  };

  public static com.google.protobuf.Parser<TMProofPathRequest> parser() {
    return PARSER;
  }

  @java.lang.Override
  public com.google.protobuf.Parser<TMProofPathRequest> getParserForType() {
    return PARSER;
  }

  @java.lang.Override
  public protocol.TMProofPathRequest getDefaultInstanceForType() {
    return DEFAULT_INSTANCE;
  }

}
