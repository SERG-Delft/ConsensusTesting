// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

/**
 * Protobuf type {@code protocol.TMReplayDeltaResponse}
 */
public final class TMReplayDeltaResponse extends
    com.google.protobuf.GeneratedMessageV3 implements
    // @@protoc_insertion_point(message_implements:protocol.TMReplayDeltaResponse)
    TMReplayDeltaResponseOrBuilder {
private static final long serialVersionUID = 0L;
  // Use TMReplayDeltaResponse.newBuilder() to construct.
  private TMReplayDeltaResponse(com.google.protobuf.GeneratedMessageV3.Builder<?> builder) {
    super(builder);
  }
  private TMReplayDeltaResponse() {
    ledgerHash_ = com.google.protobuf.ByteString.EMPTY;
    ledgerHeader_ = com.google.protobuf.ByteString.EMPTY;
    transaction_ = java.util.Collections.emptyList();
    error_ = 1;
  }

  @java.lang.Override
  @SuppressWarnings({"unused"})
  protected java.lang.Object newInstance(
      UnusedPrivateParameter unused) {
    return new TMReplayDeltaResponse();
  }

  @java.lang.Override
  public final com.google.protobuf.UnknownFieldSet
  getUnknownFields() {
    return this.unknownFields;
  }
  private TMReplayDeltaResponse(
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
            ledgerHash_ = input.readBytes();
            break;
          }
          case 18: {
            bitField0_ |= 0x00000002;
            ledgerHeader_ = input.readBytes();
            break;
          }
          case 26: {
            if (!((mutable_bitField0_ & 0x00000004) != 0)) {
              transaction_ = new java.util.ArrayList<com.google.protobuf.ByteString>();
              mutable_bitField0_ |= 0x00000004;
            }
            transaction_.add(input.readBytes());
            break;
          }
          case 32: {
            int rawValue = input.readEnum();
              @SuppressWarnings("deprecation")
            protocol.TMReplyError value = protocol.TMReplyError.valueOf(rawValue);
            if (value == null) {
              unknownFields.mergeVarintField(4, rawValue);
            } else {
              bitField0_ |= 0x00000004;
              error_ = rawValue;
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
      if (((mutable_bitField0_ & 0x00000004) != 0)) {
        transaction_ = java.util.Collections.unmodifiableList(transaction_); // C
      }
      this.unknownFields = unknownFields.build();
      makeExtensionsImmutable();
    }
  }
  public static final com.google.protobuf.Descriptors.Descriptor
      getDescriptor() {
    return protocol.Ripple.internal_static_protocol_TMReplayDeltaResponse_descriptor;
  }

  @java.lang.Override
  protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internalGetFieldAccessorTable() {
    return protocol.Ripple.internal_static_protocol_TMReplayDeltaResponse_fieldAccessorTable
        .ensureFieldAccessorsInitialized(
            protocol.TMReplayDeltaResponse.class, protocol.TMReplayDeltaResponse.Builder.class);
  }

  private int bitField0_;
  public static final int LEDGERHASH_FIELD_NUMBER = 1;
  private com.google.protobuf.ByteString ledgerHash_;
  /**
   * <code>required bytes ledgerHash = 1;</code>
   * @return Whether the ledgerHash field is set.
   */
  @java.lang.Override
  public boolean hasLedgerHash() {
    return ((bitField0_ & 0x00000001) != 0);
  }
  /**
   * <code>required bytes ledgerHash = 1;</code>
   * @return The ledgerHash.
   */
  @java.lang.Override
  public com.google.protobuf.ByteString getLedgerHash() {
    return ledgerHash_;
  }

  public static final int LEDGERHEADER_FIELD_NUMBER = 2;
  private com.google.protobuf.ByteString ledgerHeader_;
  /**
   * <code>optional bytes ledgerHeader = 2;</code>
   * @return Whether the ledgerHeader field is set.
   */
  @java.lang.Override
  public boolean hasLedgerHeader() {
    return ((bitField0_ & 0x00000002) != 0);
  }
  /**
   * <code>optional bytes ledgerHeader = 2;</code>
   * @return The ledgerHeader.
   */
  @java.lang.Override
  public com.google.protobuf.ByteString getLedgerHeader() {
    return ledgerHeader_;
  }

  public static final int TRANSACTION_FIELD_NUMBER = 3;
  private java.util.List<com.google.protobuf.ByteString> transaction_;
  /**
   * <code>repeated bytes transaction = 3;</code>
   * @return A list containing the transaction.
   */
  @java.lang.Override
  public java.util.List<com.google.protobuf.ByteString>
      getTransactionList() {
    return transaction_;
  }
  /**
   * <code>repeated bytes transaction = 3;</code>
   * @return The count of transaction.
   */
  public int getTransactionCount() {
    return transaction_.size();
  }
  /**
   * <code>repeated bytes transaction = 3;</code>
   * @param index The index of the element to return.
   * @return The transaction at the given index.
   */
  public com.google.protobuf.ByteString getTransaction(int index) {
    return transaction_.get(index);
  }

  public static final int ERROR_FIELD_NUMBER = 4;
  private int error_;
  /**
   * <code>optional .protocol.TMReplyError error = 4;</code>
   * @return Whether the error field is set.
   */
  @java.lang.Override public boolean hasError() {
    return ((bitField0_ & 0x00000004) != 0);
  }
  /**
   * <code>optional .protocol.TMReplyError error = 4;</code>
   * @return The error.
   */
  @java.lang.Override public protocol.TMReplyError getError() {
    @SuppressWarnings("deprecation")
    protocol.TMReplyError result = protocol.TMReplyError.valueOf(error_);
    return result == null ? protocol.TMReplyError.reNO_LEDGER : result;
  }

  private byte memoizedIsInitialized = -1;
  @java.lang.Override
  public final boolean isInitialized() {
    byte isInitialized = memoizedIsInitialized;
    if (isInitialized == 1) return true;
    if (isInitialized == 0) return false;

    if (!hasLedgerHash()) {
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
      output.writeBytes(1, ledgerHash_);
    }
    if (((bitField0_ & 0x00000002) != 0)) {
      output.writeBytes(2, ledgerHeader_);
    }
    for (int i = 0; i < transaction_.size(); i++) {
      output.writeBytes(3, transaction_.get(i));
    }
    if (((bitField0_ & 0x00000004) != 0)) {
      output.writeEnum(4, error_);
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
        .computeBytesSize(1, ledgerHash_);
    }
    if (((bitField0_ & 0x00000002) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeBytesSize(2, ledgerHeader_);
    }
    {
      int dataSize = 0;
      for (int i = 0; i < transaction_.size(); i++) {
        dataSize += com.google.protobuf.CodedOutputStream
          .computeBytesSizeNoTag(transaction_.get(i));
      }
      size += dataSize;
      size += 1 * getTransactionList().size();
    }
    if (((bitField0_ & 0x00000004) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeEnumSize(4, error_);
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
    if (!(obj instanceof protocol.TMReplayDeltaResponse)) {
      return super.equals(obj);
    }
    protocol.TMReplayDeltaResponse other = (protocol.TMReplayDeltaResponse) obj;

    if (hasLedgerHash() != other.hasLedgerHash()) return false;
    if (hasLedgerHash()) {
      if (!getLedgerHash()
          .equals(other.getLedgerHash())) return false;
    }
    if (hasLedgerHeader() != other.hasLedgerHeader()) return false;
    if (hasLedgerHeader()) {
      if (!getLedgerHeader()
          .equals(other.getLedgerHeader())) return false;
    }
    if (!getTransactionList()
        .equals(other.getTransactionList())) return false;
    if (hasError() != other.hasError()) return false;
    if (hasError()) {
      if (error_ != other.error_) return false;
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
    if (hasLedgerHash()) {
      hash = (37 * hash) + LEDGERHASH_FIELD_NUMBER;
      hash = (53 * hash) + getLedgerHash().hashCode();
    }
    if (hasLedgerHeader()) {
      hash = (37 * hash) + LEDGERHEADER_FIELD_NUMBER;
      hash = (53 * hash) + getLedgerHeader().hashCode();
    }
    if (getTransactionCount() > 0) {
      hash = (37 * hash) + TRANSACTION_FIELD_NUMBER;
      hash = (53 * hash) + getTransactionList().hashCode();
    }
    if (hasError()) {
      hash = (37 * hash) + ERROR_FIELD_NUMBER;
      hash = (53 * hash) + error_;
    }
    hash = (29 * hash) + unknownFields.hashCode();
    memoizedHashCode = hash;
    return hash;
  }

  public static protocol.TMReplayDeltaResponse parseFrom(
      java.nio.ByteBuffer data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMReplayDeltaResponse parseFrom(
      java.nio.ByteBuffer data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMReplayDeltaResponse parseFrom(
      com.google.protobuf.ByteString data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMReplayDeltaResponse parseFrom(
      com.google.protobuf.ByteString data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMReplayDeltaResponse parseFrom(byte[] data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMReplayDeltaResponse parseFrom(
      byte[] data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMReplayDeltaResponse parseFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMReplayDeltaResponse parseFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMReplayDeltaResponse parseDelimitedFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input);
  }
  public static protocol.TMReplayDeltaResponse parseDelimitedFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMReplayDeltaResponse parseFrom(
      com.google.protobuf.CodedInputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMReplayDeltaResponse parseFrom(
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
  public static Builder newBuilder(protocol.TMReplayDeltaResponse prototype) {
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
   * Protobuf type {@code protocol.TMReplayDeltaResponse}
   */
  public static final class Builder extends
      com.google.protobuf.GeneratedMessageV3.Builder<Builder> implements
      // @@protoc_insertion_point(builder_implements:protocol.TMReplayDeltaResponse)
      protocol.TMReplayDeltaResponseOrBuilder {
    public static final com.google.protobuf.Descriptors.Descriptor
        getDescriptor() {
      return protocol.Ripple.internal_static_protocol_TMReplayDeltaResponse_descriptor;
    }

    @java.lang.Override
    protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
        internalGetFieldAccessorTable() {
      return protocol.Ripple.internal_static_protocol_TMReplayDeltaResponse_fieldAccessorTable
          .ensureFieldAccessorsInitialized(
              protocol.TMReplayDeltaResponse.class, protocol.TMReplayDeltaResponse.Builder.class);
    }

    // Construct using protocol.TMReplayDeltaResponse.newBuilder()
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
      ledgerHash_ = com.google.protobuf.ByteString.EMPTY;
      bitField0_ = (bitField0_ & ~0x00000001);
      ledgerHeader_ = com.google.protobuf.ByteString.EMPTY;
      bitField0_ = (bitField0_ & ~0x00000002);
      transaction_ = java.util.Collections.emptyList();
      bitField0_ = (bitField0_ & ~0x00000004);
      error_ = 1;
      bitField0_ = (bitField0_ & ~0x00000008);
      return this;
    }

    @java.lang.Override
    public com.google.protobuf.Descriptors.Descriptor
        getDescriptorForType() {
      return protocol.Ripple.internal_static_protocol_TMReplayDeltaResponse_descriptor;
    }

    @java.lang.Override
    public protocol.TMReplayDeltaResponse getDefaultInstanceForType() {
      return protocol.TMReplayDeltaResponse.getDefaultInstance();
    }

    @java.lang.Override
    public protocol.TMReplayDeltaResponse build() {
      protocol.TMReplayDeltaResponse result = buildPartial();
      if (!result.isInitialized()) {
        throw newUninitializedMessageException(result);
      }
      return result;
    }

    @java.lang.Override
    public protocol.TMReplayDeltaResponse buildPartial() {
      protocol.TMReplayDeltaResponse result = new protocol.TMReplayDeltaResponse(this);
      int from_bitField0_ = bitField0_;
      int to_bitField0_ = 0;
      if (((from_bitField0_ & 0x00000001) != 0)) {
        to_bitField0_ |= 0x00000001;
      }
      result.ledgerHash_ = ledgerHash_;
      if (((from_bitField0_ & 0x00000002) != 0)) {
        to_bitField0_ |= 0x00000002;
      }
      result.ledgerHeader_ = ledgerHeader_;
      if (((bitField0_ & 0x00000004) != 0)) {
        transaction_ = java.util.Collections.unmodifiableList(transaction_);
        bitField0_ = (bitField0_ & ~0x00000004);
      }
      result.transaction_ = transaction_;
      if (((from_bitField0_ & 0x00000008) != 0)) {
        to_bitField0_ |= 0x00000004;
      }
      result.error_ = error_;
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
      if (other instanceof protocol.TMReplayDeltaResponse) {
        return mergeFrom((protocol.TMReplayDeltaResponse)other);
      } else {
        super.mergeFrom(other);
        return this;
      }
    }

    public Builder mergeFrom(protocol.TMReplayDeltaResponse other) {
      if (other == protocol.TMReplayDeltaResponse.getDefaultInstance()) return this;
      if (other.hasLedgerHash()) {
        setLedgerHash(other.getLedgerHash());
      }
      if (other.hasLedgerHeader()) {
        setLedgerHeader(other.getLedgerHeader());
      }
      if (!other.transaction_.isEmpty()) {
        if (transaction_.isEmpty()) {
          transaction_ = other.transaction_;
          bitField0_ = (bitField0_ & ~0x00000004);
        } else {
          ensureTransactionIsMutable();
          transaction_.addAll(other.transaction_);
        }
        onChanged();
      }
      if (other.hasError()) {
        setError(other.getError());
      }
      this.mergeUnknownFields(other.unknownFields);
      onChanged();
      return this;
    }

    @java.lang.Override
    public final boolean isInitialized() {
      if (!hasLedgerHash()) {
        return false;
      }
      return true;
    }

    @java.lang.Override
    public Builder mergeFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws java.io.IOException {
      protocol.TMReplayDeltaResponse parsedMessage = null;
      try {
        parsedMessage = PARSER.parsePartialFrom(input, extensionRegistry);
      } catch (com.google.protobuf.InvalidProtocolBufferException e) {
        parsedMessage = (protocol.TMReplayDeltaResponse) e.getUnfinishedMessage();
        throw e.unwrapIOException();
      } finally {
        if (parsedMessage != null) {
          mergeFrom(parsedMessage);
        }
      }
      return this;
    }
    private int bitField0_;

    private com.google.protobuf.ByteString ledgerHash_ = com.google.protobuf.ByteString.EMPTY;
    /**
     * <code>required bytes ledgerHash = 1;</code>
     * @return Whether the ledgerHash field is set.
     */
    @java.lang.Override
    public boolean hasLedgerHash() {
      return ((bitField0_ & 0x00000001) != 0);
    }
    /**
     * <code>required bytes ledgerHash = 1;</code>
     * @return The ledgerHash.
     */
    @java.lang.Override
    public com.google.protobuf.ByteString getLedgerHash() {
      return ledgerHash_;
    }
    /**
     * <code>required bytes ledgerHash = 1;</code>
     * @param value The ledgerHash to set.
     * @return This builder for chaining.
     */
    public Builder setLedgerHash(com.google.protobuf.ByteString value) {
      if (value == null) {
    throw new NullPointerException();
  }
  bitField0_ |= 0x00000001;
      ledgerHash_ = value;
      onChanged();
      return this;
    }
    /**
     * <code>required bytes ledgerHash = 1;</code>
     * @return This builder for chaining.
     */
    public Builder clearLedgerHash() {
      bitField0_ = (bitField0_ & ~0x00000001);
      ledgerHash_ = getDefaultInstance().getLedgerHash();
      onChanged();
      return this;
    }

    private com.google.protobuf.ByteString ledgerHeader_ = com.google.protobuf.ByteString.EMPTY;
    /**
     * <code>optional bytes ledgerHeader = 2;</code>
     * @return Whether the ledgerHeader field is set.
     */
    @java.lang.Override
    public boolean hasLedgerHeader() {
      return ((bitField0_ & 0x00000002) != 0);
    }
    /**
     * <code>optional bytes ledgerHeader = 2;</code>
     * @return The ledgerHeader.
     */
    @java.lang.Override
    public com.google.protobuf.ByteString getLedgerHeader() {
      return ledgerHeader_;
    }
    /**
     * <code>optional bytes ledgerHeader = 2;</code>
     * @param value The ledgerHeader to set.
     * @return This builder for chaining.
     */
    public Builder setLedgerHeader(com.google.protobuf.ByteString value) {
      if (value == null) {
    throw new NullPointerException();
  }
  bitField0_ |= 0x00000002;
      ledgerHeader_ = value;
      onChanged();
      return this;
    }
    /**
     * <code>optional bytes ledgerHeader = 2;</code>
     * @return This builder for chaining.
     */
    public Builder clearLedgerHeader() {
      bitField0_ = (bitField0_ & ~0x00000002);
      ledgerHeader_ = getDefaultInstance().getLedgerHeader();
      onChanged();
      return this;
    }

    private java.util.List<com.google.protobuf.ByteString> transaction_ = java.util.Collections.emptyList();
    private void ensureTransactionIsMutable() {
      if (!((bitField0_ & 0x00000004) != 0)) {
        transaction_ = new java.util.ArrayList<com.google.protobuf.ByteString>(transaction_);
        bitField0_ |= 0x00000004;
       }
    }
    /**
     * <code>repeated bytes transaction = 3;</code>
     * @return A list containing the transaction.
     */
    public java.util.List<com.google.protobuf.ByteString>
        getTransactionList() {
      return ((bitField0_ & 0x00000004) != 0) ?
               java.util.Collections.unmodifiableList(transaction_) : transaction_;
    }
    /**
     * <code>repeated bytes transaction = 3;</code>
     * @return The count of transaction.
     */
    public int getTransactionCount() {
      return transaction_.size();
    }
    /**
     * <code>repeated bytes transaction = 3;</code>
     * @param index The index of the element to return.
     * @return The transaction at the given index.
     */
    public com.google.protobuf.ByteString getTransaction(int index) {
      return transaction_.get(index);
    }
    /**
     * <code>repeated bytes transaction = 3;</code>
     * @param index The index to set the value at.
     * @param value The transaction to set.
     * @return This builder for chaining.
     */
    public Builder setTransaction(
        int index, com.google.protobuf.ByteString value) {
      if (value == null) {
    throw new NullPointerException();
  }
  ensureTransactionIsMutable();
      transaction_.set(index, value);
      onChanged();
      return this;
    }
    /**
     * <code>repeated bytes transaction = 3;</code>
     * @param value The transaction to add.
     * @return This builder for chaining.
     */
    public Builder addTransaction(com.google.protobuf.ByteString value) {
      if (value == null) {
    throw new NullPointerException();
  }
  ensureTransactionIsMutable();
      transaction_.add(value);
      onChanged();
      return this;
    }
    /**
     * <code>repeated bytes transaction = 3;</code>
     * @param values The transaction to add.
     * @return This builder for chaining.
     */
    public Builder addAllTransaction(
        java.lang.Iterable<? extends com.google.protobuf.ByteString> values) {
      ensureTransactionIsMutable();
      com.google.protobuf.AbstractMessageLite.Builder.addAll(
          values, transaction_);
      onChanged();
      return this;
    }
    /**
     * <code>repeated bytes transaction = 3;</code>
     * @return This builder for chaining.
     */
    public Builder clearTransaction() {
      transaction_ = java.util.Collections.emptyList();
      bitField0_ = (bitField0_ & ~0x00000004);
      onChanged();
      return this;
    }

    private int error_ = 1;
    /**
     * <code>optional .protocol.TMReplyError error = 4;</code>
     * @return Whether the error field is set.
     */
    @java.lang.Override public boolean hasError() {
      return ((bitField0_ & 0x00000008) != 0);
    }
    /**
     * <code>optional .protocol.TMReplyError error = 4;</code>
     * @return The error.
     */
    @java.lang.Override
    public protocol.TMReplyError getError() {
      @SuppressWarnings("deprecation")
      protocol.TMReplyError result = protocol.TMReplyError.valueOf(error_);
      return result == null ? protocol.TMReplyError.reNO_LEDGER : result;
    }
    /**
     * <code>optional .protocol.TMReplyError error = 4;</code>
     * @param value The error to set.
     * @return This builder for chaining.
     */
    public Builder setError(protocol.TMReplyError value) {
      if (value == null) {
        throw new NullPointerException();
      }
      bitField0_ |= 0x00000008;
      error_ = value.getNumber();
      onChanged();
      return this;
    }
    /**
     * <code>optional .protocol.TMReplyError error = 4;</code>
     * @return This builder for chaining.
     */
    public Builder clearError() {
      bitField0_ = (bitField0_ & ~0x00000008);
      error_ = 1;
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


    // @@protoc_insertion_point(builder_scope:protocol.TMReplayDeltaResponse)
  }

  // @@protoc_insertion_point(class_scope:protocol.TMReplayDeltaResponse)
  private static final protocol.TMReplayDeltaResponse DEFAULT_INSTANCE;
  static {
    DEFAULT_INSTANCE = new protocol.TMReplayDeltaResponse();
  }

  public static protocol.TMReplayDeltaResponse getDefaultInstance() {
    return DEFAULT_INSTANCE;
  }

  @java.lang.Deprecated public static final com.google.protobuf.Parser<TMReplayDeltaResponse>
      PARSER = new com.google.protobuf.AbstractParser<TMReplayDeltaResponse>() {
    @java.lang.Override
    public TMReplayDeltaResponse parsePartialFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws com.google.protobuf.InvalidProtocolBufferException {
      return new TMReplayDeltaResponse(input, extensionRegistry);
    }
  };

  public static com.google.protobuf.Parser<TMReplayDeltaResponse> parser() {
    return PARSER;
  }

  @java.lang.Override
  public com.google.protobuf.Parser<TMReplayDeltaResponse> getParserForType() {
    return PARSER;
  }

  @java.lang.Override
  public protocol.TMReplayDeltaResponse getDefaultInstanceForType() {
    return DEFAULT_INSTANCE;
  }

}

