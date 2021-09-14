// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

/**
 * <pre>
 * Info about shards held
 * </pre>
 *
 * Protobuf type {@code protocol.TMShardInfo}
 */
public final class TMShardInfo extends
    com.google.protobuf.GeneratedMessageV3 implements
    // @@protoc_insertion_point(message_implements:protocol.TMShardInfo)
    TMShardInfoOrBuilder {
private static final long serialVersionUID = 0L;
  // Use TMShardInfo.newBuilder() to construct.
  private TMShardInfo(com.google.protobuf.GeneratedMessageV3.Builder<?> builder) {
    super(builder);
  }
  private TMShardInfo() {
    shardIndexes_ = "";
    nodePubKey_ = com.google.protobuf.ByteString.EMPTY;
    endpoint_ = "";
    peerchain_ = emptyIntList();
  }

  @java.lang.Override
  @SuppressWarnings({"unused"})
  protected java.lang.Object newInstance(
      UnusedPrivateParameter unused) {
    return new TMShardInfo();
  }

  @java.lang.Override
  public final com.google.protobuf.UnknownFieldSet
  getUnknownFields() {
    return this.unknownFields;
  }
  private TMShardInfo(
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
            com.google.protobuf.ByteString bs = input.readBytes();
            bitField0_ |= 0x00000001;
            shardIndexes_ = bs;
            break;
          }
          case 18: {
            bitField0_ |= 0x00000002;
            nodePubKey_ = input.readBytes();
            break;
          }
          case 26: {
            com.google.protobuf.ByteString bs = input.readBytes();
            bitField0_ |= 0x00000004;
            endpoint_ = bs;
            break;
          }
          case 32: {
            bitField0_ |= 0x00000008;
            lastLink_ = input.readBool();
            break;
          }
          case 40: {
            if (!((mutable_bitField0_ & 0x00000010) != 0)) {
              peerchain_ = newIntList();
              mutable_bitField0_ |= 0x00000010;
            }
            peerchain_.addInt(input.readUInt32());
            break;
          }
          case 42: {
            int length = input.readRawVarint32();
            int limit = input.pushLimit(length);
            if (!((mutable_bitField0_ & 0x00000010) != 0) && input.getBytesUntilLimit() > 0) {
              peerchain_ = newIntList();
              mutable_bitField0_ |= 0x00000010;
            }
            while (input.getBytesUntilLimit() > 0) {
              peerchain_.addInt(input.readUInt32());
            }
            input.popLimit(limit);
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
      if (((mutable_bitField0_ & 0x00000010) != 0)) {
        peerchain_.makeImmutable(); // C
      }
      this.unknownFields = unknownFields.build();
      makeExtensionsImmutable();
    }
  }
  public static final com.google.protobuf.Descriptors.Descriptor
      getDescriptor() {
    return protocol.Ripple.internal_static_protocol_TMShardInfo_descriptor;
  }

  @java.lang.Override
  protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internalGetFieldAccessorTable() {
    return protocol.Ripple.internal_static_protocol_TMShardInfo_fieldAccessorTable
        .ensureFieldAccessorsInitialized(
            protocol.TMShardInfo.class, protocol.TMShardInfo.Builder.class);
  }

  private int bitField0_;
  public static final int SHARDINDEXES_FIELD_NUMBER = 1;
  private volatile java.lang.Object shardIndexes_;
  /**
   * <pre>
   * rangeSet of shard indexes
   * </pre>
   *
   * <code>required string shardIndexes = 1 [deprecated = true];</code>
   * @return Whether the shardIndexes field is set.
   */
  @java.lang.Override
  @java.lang.Deprecated public boolean hasShardIndexes() {
    return ((bitField0_ & 0x00000001) != 0);
  }
  /**
   * <pre>
   * rangeSet of shard indexes
   * </pre>
   *
   * <code>required string shardIndexes = 1 [deprecated = true];</code>
   * @return The shardIndexes.
   */
  @java.lang.Override
  @java.lang.Deprecated public java.lang.String getShardIndexes() {
    java.lang.Object ref = shardIndexes_;
    if (ref instanceof java.lang.String) {
      return (java.lang.String) ref;
    } else {
      com.google.protobuf.ByteString bs = 
          (com.google.protobuf.ByteString) ref;
      java.lang.String s = bs.toStringUtf8();
      if (bs.isValidUtf8()) {
        shardIndexes_ = s;
      }
      return s;
    }
  }
  /**
   * <pre>
   * rangeSet of shard indexes
   * </pre>
   *
   * <code>required string shardIndexes = 1 [deprecated = true];</code>
   * @return The bytes for shardIndexes.
   */
  @java.lang.Override
  @java.lang.Deprecated public com.google.protobuf.ByteString
      getShardIndexesBytes() {
    java.lang.Object ref = shardIndexes_;
    if (ref instanceof java.lang.String) {
      com.google.protobuf.ByteString b = 
          com.google.protobuf.ByteString.copyFromUtf8(
              (java.lang.String) ref);
      shardIndexes_ = b;
      return b;
    } else {
      return (com.google.protobuf.ByteString) ref;
    }
  }

  public static final int NODEPUBKEY_FIELD_NUMBER = 2;
  private com.google.protobuf.ByteString nodePubKey_;
  /**
   * <pre>
   * The node's public key
   * </pre>
   *
   * <code>optional bytes nodePubKey = 2 [deprecated = true];</code>
   * @return Whether the nodePubKey field is set.
   */
  @java.lang.Override
  @java.lang.Deprecated public boolean hasNodePubKey() {
    return ((bitField0_ & 0x00000002) != 0);
  }
  /**
   * <pre>
   * The node's public key
   * </pre>
   *
   * <code>optional bytes nodePubKey = 2 [deprecated = true];</code>
   * @return The nodePubKey.
   */
  @java.lang.Override
  @java.lang.Deprecated public com.google.protobuf.ByteString getNodePubKey() {
    return nodePubKey_;
  }

  public static final int ENDPOINT_FIELD_NUMBER = 3;
  private volatile java.lang.Object endpoint_;
  /**
   * <pre>
   * ipv6 or ipv4 address
   * </pre>
   *
   * <code>optional string endpoint = 3 [deprecated = true];</code>
   * @return Whether the endpoint field is set.
   */
  @java.lang.Override
  @java.lang.Deprecated public boolean hasEndpoint() {
    return ((bitField0_ & 0x00000004) != 0);
  }
  /**
   * <pre>
   * ipv6 or ipv4 address
   * </pre>
   *
   * <code>optional string endpoint = 3 [deprecated = true];</code>
   * @return The endpoint.
   */
  @java.lang.Override
  @java.lang.Deprecated public java.lang.String getEndpoint() {
    java.lang.Object ref = endpoint_;
    if (ref instanceof java.lang.String) {
      return (java.lang.String) ref;
    } else {
      com.google.protobuf.ByteString bs = 
          (com.google.protobuf.ByteString) ref;
      java.lang.String s = bs.toStringUtf8();
      if (bs.isValidUtf8()) {
        endpoint_ = s;
      }
      return s;
    }
  }
  /**
   * <pre>
   * ipv6 or ipv4 address
   * </pre>
   *
   * <code>optional string endpoint = 3 [deprecated = true];</code>
   * @return The bytes for endpoint.
   */
  @java.lang.Override
  @java.lang.Deprecated public com.google.protobuf.ByteString
      getEndpointBytes() {
    java.lang.Object ref = endpoint_;
    if (ref instanceof java.lang.String) {
      com.google.protobuf.ByteString b = 
          com.google.protobuf.ByteString.copyFromUtf8(
              (java.lang.String) ref);
      endpoint_ = b;
      return b;
    } else {
      return (com.google.protobuf.ByteString) ref;
    }
  }

  public static final int LASTLINK_FIELD_NUMBER = 4;
  private boolean lastLink_;
  /**
   * <pre>
   * true if last link in the peer chain
   * </pre>
   *
   * <code>optional bool lastLink = 4 [deprecated = true];</code>
   * @return Whether the lastLink field is set.
   */
  @java.lang.Override
  @java.lang.Deprecated public boolean hasLastLink() {
    return ((bitField0_ & 0x00000008) != 0);
  }
  /**
   * <pre>
   * true if last link in the peer chain
   * </pre>
   *
   * <code>optional bool lastLink = 4 [deprecated = true];</code>
   * @return The lastLink.
   */
  @java.lang.Override
  @java.lang.Deprecated public boolean getLastLink() {
    return lastLink_;
  }

  public static final int PEERCHAIN_FIELD_NUMBER = 5;
  private com.google.protobuf.Internal.IntList peerchain_;
  /**
   * <pre>
   * IDs used to route messages
   * </pre>
   *
   * <code>repeated uint32 peerchain = 5 [deprecated = true];</code>
   * @return A list containing the peerchain.
   */
  @java.lang.Override
  @java.lang.Deprecated public java.util.List<java.lang.Integer>
      getPeerchainList() {
    return peerchain_;
  }
  /**
   * <pre>
   * IDs used to route messages
   * </pre>
   *
   * <code>repeated uint32 peerchain = 5 [deprecated = true];</code>
   * @return The count of peerchain.
   */
  @java.lang.Deprecated public int getPeerchainCount() {
    return peerchain_.size();
  }
  /**
   * <pre>
   * IDs used to route messages
   * </pre>
   *
   * <code>repeated uint32 peerchain = 5 [deprecated = true];</code>
   * @param index The index of the element to return.
   * @return The peerchain at the given index.
   */
  @java.lang.Deprecated public int getPeerchain(int index) {
    return peerchain_.getInt(index);
  }

  private byte memoizedIsInitialized = -1;
  @java.lang.Override
  public final boolean isInitialized() {
    byte isInitialized = memoizedIsInitialized;
    if (isInitialized == 1) return true;
    if (isInitialized == 0) return false;

    if (!hasShardIndexes()) {
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
      com.google.protobuf.GeneratedMessageV3.writeString(output, 1, shardIndexes_);
    }
    if (((bitField0_ & 0x00000002) != 0)) {
      output.writeBytes(2, nodePubKey_);
    }
    if (((bitField0_ & 0x00000004) != 0)) {
      com.google.protobuf.GeneratedMessageV3.writeString(output, 3, endpoint_);
    }
    if (((bitField0_ & 0x00000008) != 0)) {
      output.writeBool(4, lastLink_);
    }
    for (int i = 0; i < peerchain_.size(); i++) {
      output.writeUInt32(5, peerchain_.getInt(i));
    }
    unknownFields.writeTo(output);
  }

  @java.lang.Override
  public int getSerializedSize() {
    int size = memoizedSize;
    if (size != -1) return size;

    size = 0;
    if (((bitField0_ & 0x00000001) != 0)) {
      size += com.google.protobuf.GeneratedMessageV3.computeStringSize(1, shardIndexes_);
    }
    if (((bitField0_ & 0x00000002) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeBytesSize(2, nodePubKey_);
    }
    if (((bitField0_ & 0x00000004) != 0)) {
      size += com.google.protobuf.GeneratedMessageV3.computeStringSize(3, endpoint_);
    }
    if (((bitField0_ & 0x00000008) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeBoolSize(4, lastLink_);
    }
    {
      int dataSize = 0;
      for (int i = 0; i < peerchain_.size(); i++) {
        dataSize += com.google.protobuf.CodedOutputStream
          .computeUInt32SizeNoTag(peerchain_.getInt(i));
      }
      size += dataSize;
      size += 1 * getPeerchainList().size();
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
    if (!(obj instanceof protocol.TMShardInfo)) {
      return super.equals(obj);
    }
    protocol.TMShardInfo other = (protocol.TMShardInfo) obj;

    if (hasShardIndexes() != other.hasShardIndexes()) return false;
    if (hasShardIndexes()) {
      if (!getShardIndexes()
          .equals(other.getShardIndexes())) return false;
    }
    if (hasNodePubKey() != other.hasNodePubKey()) return false;
    if (hasNodePubKey()) {
      if (!getNodePubKey()
          .equals(other.getNodePubKey())) return false;
    }
    if (hasEndpoint() != other.hasEndpoint()) return false;
    if (hasEndpoint()) {
      if (!getEndpoint()
          .equals(other.getEndpoint())) return false;
    }
    if (hasLastLink() != other.hasLastLink()) return false;
    if (hasLastLink()) {
      if (getLastLink()
          != other.getLastLink()) return false;
    }
    if (!getPeerchainList()
        .equals(other.getPeerchainList())) return false;
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
    if (hasShardIndexes()) {
      hash = (37 * hash) + SHARDINDEXES_FIELD_NUMBER;
      hash = (53 * hash) + getShardIndexes().hashCode();
    }
    if (hasNodePubKey()) {
      hash = (37 * hash) + NODEPUBKEY_FIELD_NUMBER;
      hash = (53 * hash) + getNodePubKey().hashCode();
    }
    if (hasEndpoint()) {
      hash = (37 * hash) + ENDPOINT_FIELD_NUMBER;
      hash = (53 * hash) + getEndpoint().hashCode();
    }
    if (hasLastLink()) {
      hash = (37 * hash) + LASTLINK_FIELD_NUMBER;
      hash = (53 * hash) + com.google.protobuf.Internal.hashBoolean(
          getLastLink());
    }
    if (getPeerchainCount() > 0) {
      hash = (37 * hash) + PEERCHAIN_FIELD_NUMBER;
      hash = (53 * hash) + getPeerchainList().hashCode();
    }
    hash = (29 * hash) + unknownFields.hashCode();
    memoizedHashCode = hash;
    return hash;
  }

  public static protocol.TMShardInfo parseFrom(
      java.nio.ByteBuffer data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMShardInfo parseFrom(
      java.nio.ByteBuffer data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMShardInfo parseFrom(
      com.google.protobuf.ByteString data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMShardInfo parseFrom(
      com.google.protobuf.ByteString data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMShardInfo parseFrom(byte[] data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMShardInfo parseFrom(
      byte[] data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMShardInfo parseFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMShardInfo parseFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMShardInfo parseDelimitedFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input);
  }
  public static protocol.TMShardInfo parseDelimitedFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMShardInfo parseFrom(
      com.google.protobuf.CodedInputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMShardInfo parseFrom(
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
  public static Builder newBuilder(protocol.TMShardInfo prototype) {
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
   * Info about shards held
   * </pre>
   *
   * Protobuf type {@code protocol.TMShardInfo}
   */
  public static final class Builder extends
      com.google.protobuf.GeneratedMessageV3.Builder<Builder> implements
      // @@protoc_insertion_point(builder_implements:protocol.TMShardInfo)
      protocol.TMShardInfoOrBuilder {
    public static final com.google.protobuf.Descriptors.Descriptor
        getDescriptor() {
      return protocol.Ripple.internal_static_protocol_TMShardInfo_descriptor;
    }

    @java.lang.Override
    protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
        internalGetFieldAccessorTable() {
      return protocol.Ripple.internal_static_protocol_TMShardInfo_fieldAccessorTable
          .ensureFieldAccessorsInitialized(
              protocol.TMShardInfo.class, protocol.TMShardInfo.Builder.class);
    }

    // Construct using protocol.TMShardInfo.newBuilder()
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
      shardIndexes_ = "";
      bitField0_ = (bitField0_ & ~0x00000001);
      nodePubKey_ = com.google.protobuf.ByteString.EMPTY;
      bitField0_ = (bitField0_ & ~0x00000002);
      endpoint_ = "";
      bitField0_ = (bitField0_ & ~0x00000004);
      lastLink_ = false;
      bitField0_ = (bitField0_ & ~0x00000008);
      peerchain_ = emptyIntList();
      bitField0_ = (bitField0_ & ~0x00000010);
      return this;
    }

    @java.lang.Override
    public com.google.protobuf.Descriptors.Descriptor
        getDescriptorForType() {
      return protocol.Ripple.internal_static_protocol_TMShardInfo_descriptor;
    }

    @java.lang.Override
    public protocol.TMShardInfo getDefaultInstanceForType() {
      return protocol.TMShardInfo.getDefaultInstance();
    }

    @java.lang.Override
    public protocol.TMShardInfo build() {
      protocol.TMShardInfo result = buildPartial();
      if (!result.isInitialized()) {
        throw newUninitializedMessageException(result);
      }
      return result;
    }

    @java.lang.Override
    public protocol.TMShardInfo buildPartial() {
      protocol.TMShardInfo result = new protocol.TMShardInfo(this);
      int from_bitField0_ = bitField0_;
      int to_bitField0_ = 0;
      if (((from_bitField0_ & 0x00000001) != 0)) {
        to_bitField0_ |= 0x00000001;
      }
      result.shardIndexes_ = shardIndexes_;
      if (((from_bitField0_ & 0x00000002) != 0)) {
        to_bitField0_ |= 0x00000002;
      }
      result.nodePubKey_ = nodePubKey_;
      if (((from_bitField0_ & 0x00000004) != 0)) {
        to_bitField0_ |= 0x00000004;
      }
      result.endpoint_ = endpoint_;
      if (((from_bitField0_ & 0x00000008) != 0)) {
        result.lastLink_ = lastLink_;
        to_bitField0_ |= 0x00000008;
      }
      if (((bitField0_ & 0x00000010) != 0)) {
        peerchain_.makeImmutable();
        bitField0_ = (bitField0_ & ~0x00000010);
      }
      result.peerchain_ = peerchain_;
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
      if (other instanceof protocol.TMShardInfo) {
        return mergeFrom((protocol.TMShardInfo)other);
      } else {
        super.mergeFrom(other);
        return this;
      }
    }

    public Builder mergeFrom(protocol.TMShardInfo other) {
      if (other == protocol.TMShardInfo.getDefaultInstance()) return this;
      if (other.hasShardIndexes()) {
        bitField0_ |= 0x00000001;
        shardIndexes_ = other.shardIndexes_;
        onChanged();
      }
      if (other.hasNodePubKey()) {
        setNodePubKey(other.getNodePubKey());
      }
      if (other.hasEndpoint()) {
        bitField0_ |= 0x00000004;
        endpoint_ = other.endpoint_;
        onChanged();
      }
      if (other.hasLastLink()) {
        setLastLink(other.getLastLink());
      }
      if (!other.peerchain_.isEmpty()) {
        if (peerchain_.isEmpty()) {
          peerchain_ = other.peerchain_;
          bitField0_ = (bitField0_ & ~0x00000010);
        } else {
          ensurePeerchainIsMutable();
          peerchain_.addAll(other.peerchain_);
        }
        onChanged();
      }
      this.mergeUnknownFields(other.unknownFields);
      onChanged();
      return this;
    }

    @java.lang.Override
    public final boolean isInitialized() {
      if (!hasShardIndexes()) {
        return false;
      }
      return true;
    }

    @java.lang.Override
    public Builder mergeFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws java.io.IOException {
      protocol.TMShardInfo parsedMessage = null;
      try {
        parsedMessage = PARSER.parsePartialFrom(input, extensionRegistry);
      } catch (com.google.protobuf.InvalidProtocolBufferException e) {
        parsedMessage = (protocol.TMShardInfo) e.getUnfinishedMessage();
        throw e.unwrapIOException();
      } finally {
        if (parsedMessage != null) {
          mergeFrom(parsedMessage);
        }
      }
      return this;
    }
    private int bitField0_;

    private java.lang.Object shardIndexes_ = "";
    /**
     * <pre>
     * rangeSet of shard indexes
     * </pre>
     *
     * <code>required string shardIndexes = 1 [deprecated = true];</code>
     * @return Whether the shardIndexes field is set.
     */
    @java.lang.Deprecated public boolean hasShardIndexes() {
      return ((bitField0_ & 0x00000001) != 0);
    }
    /**
     * <pre>
     * rangeSet of shard indexes
     * </pre>
     *
     * <code>required string shardIndexes = 1 [deprecated = true];</code>
     * @return The shardIndexes.
     */
    @java.lang.Deprecated public java.lang.String getShardIndexes() {
      java.lang.Object ref = shardIndexes_;
      if (!(ref instanceof java.lang.String)) {
        com.google.protobuf.ByteString bs =
            (com.google.protobuf.ByteString) ref;
        java.lang.String s = bs.toStringUtf8();
        if (bs.isValidUtf8()) {
          shardIndexes_ = s;
        }
        return s;
      } else {
        return (java.lang.String) ref;
      }
    }
    /**
     * <pre>
     * rangeSet of shard indexes
     * </pre>
     *
     * <code>required string shardIndexes = 1 [deprecated = true];</code>
     * @return The bytes for shardIndexes.
     */
    @java.lang.Deprecated public com.google.protobuf.ByteString
        getShardIndexesBytes() {
      java.lang.Object ref = shardIndexes_;
      if (ref instanceof String) {
        com.google.protobuf.ByteString b = 
            com.google.protobuf.ByteString.copyFromUtf8(
                (java.lang.String) ref);
        shardIndexes_ = b;
        return b;
      } else {
        return (com.google.protobuf.ByteString) ref;
      }
    }
    /**
     * <pre>
     * rangeSet of shard indexes
     * </pre>
     *
     * <code>required string shardIndexes = 1 [deprecated = true];</code>
     * @param value The shardIndexes to set.
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder setShardIndexes(
        java.lang.String value) {
      if (value == null) {
    throw new NullPointerException();
  }
  bitField0_ |= 0x00000001;
      shardIndexes_ = value;
      onChanged();
      return this;
    }
    /**
     * <pre>
     * rangeSet of shard indexes
     * </pre>
     *
     * <code>required string shardIndexes = 1 [deprecated = true];</code>
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder clearShardIndexes() {
      bitField0_ = (bitField0_ & ~0x00000001);
      shardIndexes_ = getDefaultInstance().getShardIndexes();
      onChanged();
      return this;
    }
    /**
     * <pre>
     * rangeSet of shard indexes
     * </pre>
     *
     * <code>required string shardIndexes = 1 [deprecated = true];</code>
     * @param value The bytes for shardIndexes to set.
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder setShardIndexesBytes(
        com.google.protobuf.ByteString value) {
      if (value == null) {
    throw new NullPointerException();
  }
  bitField0_ |= 0x00000001;
      shardIndexes_ = value;
      onChanged();
      return this;
    }

    private com.google.protobuf.ByteString nodePubKey_ = com.google.protobuf.ByteString.EMPTY;
    /**
     * <pre>
     * The node's public key
     * </pre>
     *
     * <code>optional bytes nodePubKey = 2 [deprecated = true];</code>
     * @return Whether the nodePubKey field is set.
     */
    @java.lang.Override
    @java.lang.Deprecated public boolean hasNodePubKey() {
      return ((bitField0_ & 0x00000002) != 0);
    }
    /**
     * <pre>
     * The node's public key
     * </pre>
     *
     * <code>optional bytes nodePubKey = 2 [deprecated = true];</code>
     * @return The nodePubKey.
     */
    @java.lang.Override
    @java.lang.Deprecated public com.google.protobuf.ByteString getNodePubKey() {
      return nodePubKey_;
    }
    /**
     * <pre>
     * The node's public key
     * </pre>
     *
     * <code>optional bytes nodePubKey = 2 [deprecated = true];</code>
     * @param value The nodePubKey to set.
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder setNodePubKey(com.google.protobuf.ByteString value) {
      if (value == null) {
    throw new NullPointerException();
  }
  bitField0_ |= 0x00000002;
      nodePubKey_ = value;
      onChanged();
      return this;
    }
    /**
     * <pre>
     * The node's public key
     * </pre>
     *
     * <code>optional bytes nodePubKey = 2 [deprecated = true];</code>
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder clearNodePubKey() {
      bitField0_ = (bitField0_ & ~0x00000002);
      nodePubKey_ = getDefaultInstance().getNodePubKey();
      onChanged();
      return this;
    }

    private java.lang.Object endpoint_ = "";
    /**
     * <pre>
     * ipv6 or ipv4 address
     * </pre>
     *
     * <code>optional string endpoint = 3 [deprecated = true];</code>
     * @return Whether the endpoint field is set.
     */
    @java.lang.Deprecated public boolean hasEndpoint() {
      return ((bitField0_ & 0x00000004) != 0);
    }
    /**
     * <pre>
     * ipv6 or ipv4 address
     * </pre>
     *
     * <code>optional string endpoint = 3 [deprecated = true];</code>
     * @return The endpoint.
     */
    @java.lang.Deprecated public java.lang.String getEndpoint() {
      java.lang.Object ref = endpoint_;
      if (!(ref instanceof java.lang.String)) {
        com.google.protobuf.ByteString bs =
            (com.google.protobuf.ByteString) ref;
        java.lang.String s = bs.toStringUtf8();
        if (bs.isValidUtf8()) {
          endpoint_ = s;
        }
        return s;
      } else {
        return (java.lang.String) ref;
      }
    }
    /**
     * <pre>
     * ipv6 or ipv4 address
     * </pre>
     *
     * <code>optional string endpoint = 3 [deprecated = true];</code>
     * @return The bytes for endpoint.
     */
    @java.lang.Deprecated public com.google.protobuf.ByteString
        getEndpointBytes() {
      java.lang.Object ref = endpoint_;
      if (ref instanceof String) {
        com.google.protobuf.ByteString b = 
            com.google.protobuf.ByteString.copyFromUtf8(
                (java.lang.String) ref);
        endpoint_ = b;
        return b;
      } else {
        return (com.google.protobuf.ByteString) ref;
      }
    }
    /**
     * <pre>
     * ipv6 or ipv4 address
     * </pre>
     *
     * <code>optional string endpoint = 3 [deprecated = true];</code>
     * @param value The endpoint to set.
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder setEndpoint(
        java.lang.String value) {
      if (value == null) {
    throw new NullPointerException();
  }
  bitField0_ |= 0x00000004;
      endpoint_ = value;
      onChanged();
      return this;
    }
    /**
     * <pre>
     * ipv6 or ipv4 address
     * </pre>
     *
     * <code>optional string endpoint = 3 [deprecated = true];</code>
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder clearEndpoint() {
      bitField0_ = (bitField0_ & ~0x00000004);
      endpoint_ = getDefaultInstance().getEndpoint();
      onChanged();
      return this;
    }
    /**
     * <pre>
     * ipv6 or ipv4 address
     * </pre>
     *
     * <code>optional string endpoint = 3 [deprecated = true];</code>
     * @param value The bytes for endpoint to set.
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder setEndpointBytes(
        com.google.protobuf.ByteString value) {
      if (value == null) {
    throw new NullPointerException();
  }
  bitField0_ |= 0x00000004;
      endpoint_ = value;
      onChanged();
      return this;
    }

    private boolean lastLink_ ;
    /**
     * <pre>
     * true if last link in the peer chain
     * </pre>
     *
     * <code>optional bool lastLink = 4 [deprecated = true];</code>
     * @return Whether the lastLink field is set.
     */
    @java.lang.Override
    @java.lang.Deprecated public boolean hasLastLink() {
      return ((bitField0_ & 0x00000008) != 0);
    }
    /**
     * <pre>
     * true if last link in the peer chain
     * </pre>
     *
     * <code>optional bool lastLink = 4 [deprecated = true];</code>
     * @return The lastLink.
     */
    @java.lang.Override
    @java.lang.Deprecated public boolean getLastLink() {
      return lastLink_;
    }
    /**
     * <pre>
     * true if last link in the peer chain
     * </pre>
     *
     * <code>optional bool lastLink = 4 [deprecated = true];</code>
     * @param value The lastLink to set.
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder setLastLink(boolean value) {
      bitField0_ |= 0x00000008;
      lastLink_ = value;
      onChanged();
      return this;
    }
    /**
     * <pre>
     * true if last link in the peer chain
     * </pre>
     *
     * <code>optional bool lastLink = 4 [deprecated = true];</code>
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder clearLastLink() {
      bitField0_ = (bitField0_ & ~0x00000008);
      lastLink_ = false;
      onChanged();
      return this;
    }

    private com.google.protobuf.Internal.IntList peerchain_ = emptyIntList();
    private void ensurePeerchainIsMutable() {
      if (!((bitField0_ & 0x00000010) != 0)) {
        peerchain_ = mutableCopy(peerchain_);
        bitField0_ |= 0x00000010;
       }
    }
    /**
     * <pre>
     * IDs used to route messages
     * </pre>
     *
     * <code>repeated uint32 peerchain = 5 [deprecated = true];</code>
     * @return A list containing the peerchain.
     */
    @java.lang.Deprecated public java.util.List<java.lang.Integer>
        getPeerchainList() {
      return ((bitField0_ & 0x00000010) != 0) ?
               java.util.Collections.unmodifiableList(peerchain_) : peerchain_;
    }
    /**
     * <pre>
     * IDs used to route messages
     * </pre>
     *
     * <code>repeated uint32 peerchain = 5 [deprecated = true];</code>
     * @return The count of peerchain.
     */
    @java.lang.Deprecated public int getPeerchainCount() {
      return peerchain_.size();
    }
    /**
     * <pre>
     * IDs used to route messages
     * </pre>
     *
     * <code>repeated uint32 peerchain = 5 [deprecated = true];</code>
     * @param index The index of the element to return.
     * @return The peerchain at the given index.
     */
    @java.lang.Deprecated public int getPeerchain(int index) {
      return peerchain_.getInt(index);
    }
    /**
     * <pre>
     * IDs used to route messages
     * </pre>
     *
     * <code>repeated uint32 peerchain = 5 [deprecated = true];</code>
     * @param index The index to set the value at.
     * @param value The peerchain to set.
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder setPeerchain(
        int index, int value) {
      ensurePeerchainIsMutable();
      peerchain_.setInt(index, value);
      onChanged();
      return this;
    }
    /**
     * <pre>
     * IDs used to route messages
     * </pre>
     *
     * <code>repeated uint32 peerchain = 5 [deprecated = true];</code>
     * @param value The peerchain to add.
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder addPeerchain(int value) {
      ensurePeerchainIsMutable();
      peerchain_.addInt(value);
      onChanged();
      return this;
    }
    /**
     * <pre>
     * IDs used to route messages
     * </pre>
     *
     * <code>repeated uint32 peerchain = 5 [deprecated = true];</code>
     * @param values The peerchain to add.
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder addAllPeerchain(
        java.lang.Iterable<? extends java.lang.Integer> values) {
      ensurePeerchainIsMutable();
      com.google.protobuf.AbstractMessageLite.Builder.addAll(
          values, peerchain_);
      onChanged();
      return this;
    }
    /**
     * <pre>
     * IDs used to route messages
     * </pre>
     *
     * <code>repeated uint32 peerchain = 5 [deprecated = true];</code>
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder clearPeerchain() {
      peerchain_ = emptyIntList();
      bitField0_ = (bitField0_ & ~0x00000010);
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


    // @@protoc_insertion_point(builder_scope:protocol.TMShardInfo)
  }

  // @@protoc_insertion_point(class_scope:protocol.TMShardInfo)
  private static final protocol.TMShardInfo DEFAULT_INSTANCE;
  static {
    DEFAULT_INSTANCE = new protocol.TMShardInfo();
  }

  public static protocol.TMShardInfo getDefaultInstance() {
    return DEFAULT_INSTANCE;
  }

  @java.lang.Deprecated public static final com.google.protobuf.Parser<TMShardInfo>
      PARSER = new com.google.protobuf.AbstractParser<TMShardInfo>() {
    @java.lang.Override
    public TMShardInfo parsePartialFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws com.google.protobuf.InvalidProtocolBufferException {
      return new TMShardInfo(input, extensionRegistry);
    }
  };

  public static com.google.protobuf.Parser<TMShardInfo> parser() {
    return PARSER;
  }

  @java.lang.Override
  public com.google.protobuf.Parser<TMShardInfo> getParserForType() {
    return PARSER;
  }

  @java.lang.Override
  public protocol.TMShardInfo getDefaultInstanceForType() {
    return DEFAULT_INSTANCE;
  }

}

