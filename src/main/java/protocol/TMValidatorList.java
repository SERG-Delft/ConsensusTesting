// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

/**
 * <pre>
 * Validator list (UNL)
 * </pre>
 *
 * Protobuf type {@code protocol.TMValidatorList}
 */
public final class TMValidatorList extends
    com.google.protobuf.GeneratedMessageV3 implements
    // @@protoc_insertion_point(message_implements:protocol.TMValidatorList)
    TMValidatorListOrBuilder {
private static final long serialVersionUID = 0L;
  // Use TMValidatorList.newBuilder() to construct.
  private TMValidatorList(com.google.protobuf.GeneratedMessageV3.Builder<?> builder) {
    super(builder);
  }
  private TMValidatorList() {
    manifest_ = com.google.protobuf.ByteString.EMPTY;
    blob_ = com.google.protobuf.ByteString.EMPTY;
    signature_ = com.google.protobuf.ByteString.EMPTY;
  }

  @java.lang.Override
  @SuppressWarnings({"unused"})
  protected java.lang.Object newInstance(
      UnusedPrivateParameter unused) {
    return new TMValidatorList();
  }

  @java.lang.Override
  public final com.google.protobuf.UnknownFieldSet
  getUnknownFields() {
    return this.unknownFields;
  }
  private TMValidatorList(
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
            manifest_ = input.readBytes();
            break;
          }
          case 18: {
            bitField0_ |= 0x00000002;
            blob_ = input.readBytes();
            break;
          }
          case 26: {
            bitField0_ |= 0x00000004;
            signature_ = input.readBytes();
            break;
          }
          case 32: {
            bitField0_ |= 0x00000008;
            version_ = input.readUInt32();
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
    return protocol.Ripple.internal_static_protocol_TMValidatorList_descriptor;
  }

  @java.lang.Override
  protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internalGetFieldAccessorTable() {
    return protocol.Ripple.internal_static_protocol_TMValidatorList_fieldAccessorTable
        .ensureFieldAccessorsInitialized(
            protocol.TMValidatorList.class, protocol.TMValidatorList.Builder.class);
  }

  private int bitField0_;
  public static final int MANIFEST_FIELD_NUMBER = 1;
  private com.google.protobuf.ByteString manifest_;
  /**
   * <code>required bytes manifest = 1;</code>
   * @return Whether the manifest field is set.
   */
  @java.lang.Override
  public boolean hasManifest() {
    return ((bitField0_ & 0x00000001) != 0);
  }
  /**
   * <code>required bytes manifest = 1;</code>
   * @return The manifest.
   */
  @java.lang.Override
  public com.google.protobuf.ByteString getManifest() {
    return manifest_;
  }

  public static final int BLOB_FIELD_NUMBER = 2;
  private com.google.protobuf.ByteString blob_;
  /**
   * <code>required bytes blob = 2;</code>
   * @return Whether the blob field is set.
   */
  @java.lang.Override
  public boolean hasBlob() {
    return ((bitField0_ & 0x00000002) != 0);
  }
  /**
   * <code>required bytes blob = 2;</code>
   * @return The blob.
   */
  @java.lang.Override
  public com.google.protobuf.ByteString getBlob() {
    return blob_;
  }

  public static final int SIGNATURE_FIELD_NUMBER = 3;
  private com.google.protobuf.ByteString signature_;
  /**
   * <code>required bytes signature = 3;</code>
   * @return Whether the signature field is set.
   */
  @java.lang.Override
  public boolean hasSignature() {
    return ((bitField0_ & 0x00000004) != 0);
  }
  /**
   * <code>required bytes signature = 3;</code>
   * @return The signature.
   */
  @java.lang.Override
  public com.google.protobuf.ByteString getSignature() {
    return signature_;
  }

  public static final int VERSION_FIELD_NUMBER = 4;
  private int version_;
  /**
   * <code>required uint32 version = 4;</code>
   * @return Whether the version field is set.
   */
  @java.lang.Override
  public boolean hasVersion() {
    return ((bitField0_ & 0x00000008) != 0);
  }
  /**
   * <code>required uint32 version = 4;</code>
   * @return The version.
   */
  @java.lang.Override
  public int getVersion() {
    return version_;
  }

  private byte memoizedIsInitialized = -1;
  @java.lang.Override
  public final boolean isInitialized() {
    byte isInitialized = memoizedIsInitialized;
    if (isInitialized == 1) return true;
    if (isInitialized == 0) return false;

    if (!hasManifest()) {
      memoizedIsInitialized = 0;
      return false;
    }
    if (!hasBlob()) {
      memoizedIsInitialized = 0;
      return false;
    }
    if (!hasSignature()) {
      memoizedIsInitialized = 0;
      return false;
    }
    if (!hasVersion()) {
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
      output.writeBytes(1, manifest_);
    }
    if (((bitField0_ & 0x00000002) != 0)) {
      output.writeBytes(2, blob_);
    }
    if (((bitField0_ & 0x00000004) != 0)) {
      output.writeBytes(3, signature_);
    }
    if (((bitField0_ & 0x00000008) != 0)) {
      output.writeUInt32(4, version_);
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
        .computeBytesSize(1, manifest_);
    }
    if (((bitField0_ & 0x00000002) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeBytesSize(2, blob_);
    }
    if (((bitField0_ & 0x00000004) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeBytesSize(3, signature_);
    }
    if (((bitField0_ & 0x00000008) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeUInt32Size(4, version_);
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
    if (!(obj instanceof protocol.TMValidatorList)) {
      return super.equals(obj);
    }
    protocol.TMValidatorList other = (protocol.TMValidatorList) obj;

    if (hasManifest() != other.hasManifest()) return false;
    if (hasManifest()) {
      if (!getManifest()
          .equals(other.getManifest())) return false;
    }
    if (hasBlob() != other.hasBlob()) return false;
    if (hasBlob()) {
      if (!getBlob()
          .equals(other.getBlob())) return false;
    }
    if (hasSignature() != other.hasSignature()) return false;
    if (hasSignature()) {
      if (!getSignature()
          .equals(other.getSignature())) return false;
    }
    if (hasVersion() != other.hasVersion()) return false;
    if (hasVersion()) {
      if (getVersion()
          != other.getVersion()) return false;
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
    if (hasManifest()) {
      hash = (37 * hash) + MANIFEST_FIELD_NUMBER;
      hash = (53 * hash) + getManifest().hashCode();
    }
    if (hasBlob()) {
      hash = (37 * hash) + BLOB_FIELD_NUMBER;
      hash = (53 * hash) + getBlob().hashCode();
    }
    if (hasSignature()) {
      hash = (37 * hash) + SIGNATURE_FIELD_NUMBER;
      hash = (53 * hash) + getSignature().hashCode();
    }
    if (hasVersion()) {
      hash = (37 * hash) + VERSION_FIELD_NUMBER;
      hash = (53 * hash) + getVersion();
    }
    hash = (29 * hash) + unknownFields.hashCode();
    memoizedHashCode = hash;
    return hash;
  }

  public static protocol.TMValidatorList parseFrom(
      java.nio.ByteBuffer data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMValidatorList parseFrom(
      java.nio.ByteBuffer data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMValidatorList parseFrom(
      com.google.protobuf.ByteString data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMValidatorList parseFrom(
      com.google.protobuf.ByteString data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMValidatorList parseFrom(byte[] data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMValidatorList parseFrom(
      byte[] data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMValidatorList parseFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMValidatorList parseFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMValidatorList parseDelimitedFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input);
  }
  public static protocol.TMValidatorList parseDelimitedFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMValidatorList parseFrom(
      com.google.protobuf.CodedInputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMValidatorList parseFrom(
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
  public static Builder newBuilder(protocol.TMValidatorList prototype) {
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
   * <pre>
   * Validator list (UNL)
   * </pre>
   *
   * Protobuf type {@code protocol.TMValidatorList}
   */
  public static final class Builder extends
      com.google.protobuf.GeneratedMessageV3.Builder<Builder> implements
      // @@protoc_insertion_point(builder_implements:protocol.TMValidatorList)
      protocol.TMValidatorListOrBuilder {
    public static final com.google.protobuf.Descriptors.Descriptor
        getDescriptor() {
      return protocol.Ripple.internal_static_protocol_TMValidatorList_descriptor;
    }

    @java.lang.Override
    protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
        internalGetFieldAccessorTable() {
      return protocol.Ripple.internal_static_protocol_TMValidatorList_fieldAccessorTable
          .ensureFieldAccessorsInitialized(
              protocol.TMValidatorList.class, protocol.TMValidatorList.Builder.class);
    }

    // Construct using protocol.TMValidatorList.newBuilder()
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
      manifest_ = com.google.protobuf.ByteString.EMPTY;
      bitField0_ = (bitField0_ & ~0x00000001);
      blob_ = com.google.protobuf.ByteString.EMPTY;
      bitField0_ = (bitField0_ & ~0x00000002);
      signature_ = com.google.protobuf.ByteString.EMPTY;
      bitField0_ = (bitField0_ & ~0x00000004);
      version_ = 0;
      bitField0_ = (bitField0_ & ~0x00000008);
      return this;
    }

    @java.lang.Override
    public com.google.protobuf.Descriptors.Descriptor
        getDescriptorForType() {
      return protocol.Ripple.internal_static_protocol_TMValidatorList_descriptor;
    }

    @java.lang.Override
    public protocol.TMValidatorList getDefaultInstanceForType() {
      return protocol.TMValidatorList.getDefaultInstance();
    }

    @java.lang.Override
    public protocol.TMValidatorList build() {
      protocol.TMValidatorList result = buildPartial();
      if (!result.isInitialized()) {
        throw newUninitializedMessageException(result);
      }
      return result;
    }

    @java.lang.Override
    public protocol.TMValidatorList buildPartial() {
      protocol.TMValidatorList result = new protocol.TMValidatorList(this);
      int from_bitField0_ = bitField0_;
      int to_bitField0_ = 0;
      if (((from_bitField0_ & 0x00000001) != 0)) {
        to_bitField0_ |= 0x00000001;
      }
      result.manifest_ = manifest_;
      if (((from_bitField0_ & 0x00000002) != 0)) {
        to_bitField0_ |= 0x00000002;
      }
      result.blob_ = blob_;
      if (((from_bitField0_ & 0x00000004) != 0)) {
        to_bitField0_ |= 0x00000004;
      }
      result.signature_ = signature_;
      if (((from_bitField0_ & 0x00000008) != 0)) {
        result.version_ = version_;
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
      if (other instanceof protocol.TMValidatorList) {
        return mergeFrom((protocol.TMValidatorList)other);
      } else {
        super.mergeFrom(other);
        return this;
      }
    }

    public Builder mergeFrom(protocol.TMValidatorList other) {
      if (other == protocol.TMValidatorList.getDefaultInstance()) return this;
      if (other.hasManifest()) {
        setManifest(other.getManifest());
      }
      if (other.hasBlob()) {
        setBlob(other.getBlob());
      }
      if (other.hasSignature()) {
        setSignature(other.getSignature());
      }
      if (other.hasVersion()) {
        setVersion(other.getVersion());
      }
      this.mergeUnknownFields(other.unknownFields);
      onChanged();
      return this;
    }

    @java.lang.Override
    public final boolean isInitialized() {
      if (!hasManifest()) {
        return false;
      }
      if (!hasBlob()) {
        return false;
      }
      if (!hasSignature()) {
        return false;
      }
      if (!hasVersion()) {
        return false;
      }
      return true;
    }

    @java.lang.Override
    public Builder mergeFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws java.io.IOException {
      protocol.TMValidatorList parsedMessage = null;
      try {
        parsedMessage = PARSER.parsePartialFrom(input, extensionRegistry);
      } catch (com.google.protobuf.InvalidProtocolBufferException e) {
        parsedMessage = (protocol.TMValidatorList) e.getUnfinishedMessage();
        throw e.unwrapIOException();
      } finally {
        if (parsedMessage != null) {
          mergeFrom(parsedMessage);
        }
      }
      return this;
    }
    private int bitField0_;

    private com.google.protobuf.ByteString manifest_ = com.google.protobuf.ByteString.EMPTY;
    /**
     * <code>required bytes manifest = 1;</code>
     * @return Whether the manifest field is set.
     */
    @java.lang.Override
    public boolean hasManifest() {
      return ((bitField0_ & 0x00000001) != 0);
    }
    /**
     * <code>required bytes manifest = 1;</code>
     * @return The manifest.
     */
    @java.lang.Override
    public com.google.protobuf.ByteString getManifest() {
      return manifest_;
    }
    /**
     * <code>required bytes manifest = 1;</code>
     * @param value The manifest to set.
     * @return This builder for chaining.
     */
    public Builder setManifest(com.google.protobuf.ByteString value) {
      if (value == null) {
    throw new NullPointerException();
  }
  bitField0_ |= 0x00000001;
      manifest_ = value;
      onChanged();
      return this;
    }
    /**
     * <code>required bytes manifest = 1;</code>
     * @return This builder for chaining.
     */
    public Builder clearManifest() {
      bitField0_ = (bitField0_ & ~0x00000001);
      manifest_ = getDefaultInstance().getManifest();
      onChanged();
      return this;
    }

    private com.google.protobuf.ByteString blob_ = com.google.protobuf.ByteString.EMPTY;
    /**
     * <code>required bytes blob = 2;</code>
     * @return Whether the blob field is set.
     */
    @java.lang.Override
    public boolean hasBlob() {
      return ((bitField0_ & 0x00000002) != 0);
    }
    /**
     * <code>required bytes blob = 2;</code>
     * @return The blob.
     */
    @java.lang.Override
    public com.google.protobuf.ByteString getBlob() {
      return blob_;
    }
    /**
     * <code>required bytes blob = 2;</code>
     * @param value The blob to set.
     * @return This builder for chaining.
     */
    public Builder setBlob(com.google.protobuf.ByteString value) {
      if (value == null) {
    throw new NullPointerException();
  }
  bitField0_ |= 0x00000002;
      blob_ = value;
      onChanged();
      return this;
    }
    /**
     * <code>required bytes blob = 2;</code>
     * @return This builder for chaining.
     */
    public Builder clearBlob() {
      bitField0_ = (bitField0_ & ~0x00000002);
      blob_ = getDefaultInstance().getBlob();
      onChanged();
      return this;
    }

    private com.google.protobuf.ByteString signature_ = com.google.protobuf.ByteString.EMPTY;
    /**
     * <code>required bytes signature = 3;</code>
     * @return Whether the signature field is set.
     */
    @java.lang.Override
    public boolean hasSignature() {
      return ((bitField0_ & 0x00000004) != 0);
    }
    /**
     * <code>required bytes signature = 3;</code>
     * @return The signature.
     */
    @java.lang.Override
    public com.google.protobuf.ByteString getSignature() {
      return signature_;
    }
    /**
     * <code>required bytes signature = 3;</code>
     * @param value The signature to set.
     * @return This builder for chaining.
     */
    public Builder setSignature(com.google.protobuf.ByteString value) {
      if (value == null) {
    throw new NullPointerException();
  }
  bitField0_ |= 0x00000004;
      signature_ = value;
      onChanged();
      return this;
    }
    /**
     * <code>required bytes signature = 3;</code>
     * @return This builder for chaining.
     */
    public Builder clearSignature() {
      bitField0_ = (bitField0_ & ~0x00000004);
      signature_ = getDefaultInstance().getSignature();
      onChanged();
      return this;
    }

    private int version_ ;
    /**
     * <code>required uint32 version = 4;</code>
     * @return Whether the version field is set.
     */
    @java.lang.Override
    public boolean hasVersion() {
      return ((bitField0_ & 0x00000008) != 0);
    }
    /**
     * <code>required uint32 version = 4;</code>
     * @return The version.
     */
    @java.lang.Override
    public int getVersion() {
      return version_;
    }
    /**
     * <code>required uint32 version = 4;</code>
     * @param value The version to set.
     * @return This builder for chaining.
     */
    public Builder setVersion(int value) {
      bitField0_ |= 0x00000008;
      version_ = value;
      onChanged();
      return this;
    }
    /**
     * <code>required uint32 version = 4;</code>
     * @return This builder for chaining.
     */
    public Builder clearVersion() {
      bitField0_ = (bitField0_ & ~0x00000008);
      version_ = 0;
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


    // @@protoc_insertion_point(builder_scope:protocol.TMValidatorList)
  }

  // @@protoc_insertion_point(class_scope:protocol.TMValidatorList)
  private static final protocol.TMValidatorList DEFAULT_INSTANCE;
  static {
    DEFAULT_INSTANCE = new protocol.TMValidatorList();
  }

  public static protocol.TMValidatorList getDefaultInstance() {
    return DEFAULT_INSTANCE;
  }

  @java.lang.Deprecated public static final com.google.protobuf.Parser<TMValidatorList>
      PARSER = new com.google.protobuf.AbstractParser<TMValidatorList>() {
    @java.lang.Override
    public TMValidatorList parsePartialFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws com.google.protobuf.InvalidProtocolBufferException {
      return new TMValidatorList(input, extensionRegistry);
    }
  };

  public static com.google.protobuf.Parser<TMValidatorList> parser() {
    return PARSER;
  }

  @java.lang.Override
  public com.google.protobuf.Parser<TMValidatorList> getParserForType() {
    return PARSER;
  }

  @java.lang.Override
  public protocol.TMValidatorList getDefaultInstanceForType() {
    return DEFAULT_INSTANCE;
  }

}

