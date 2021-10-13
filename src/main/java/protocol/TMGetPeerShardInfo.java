// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

/**
 * <pre>
 * Request info on shards held
 * </pre>
 *
 * Protobuf type {@code protocol.TMGetPeerShardInfo}
 */
public final class TMGetPeerShardInfo extends
    com.google.protobuf.GeneratedMessageV3 implements
    // @@protoc_insertion_point(message_implements:protocol.TMGetPeerShardInfo)
    TMGetPeerShardInfoOrBuilder {
private static final long serialVersionUID = 0L;
  // Use TMGetPeerShardInfo.newBuilder() to construct.
  private TMGetPeerShardInfo(com.google.protobuf.GeneratedMessageV3.Builder<?> builder) {
    super(builder);
  }
  private TMGetPeerShardInfo() {
    peerChain_ = java.util.Collections.emptyList();
  }

  @java.lang.Override
  @SuppressWarnings({"unused"})
  protected java.lang.Object newInstance(
      UnusedPrivateParameter unused) {
    return new TMGetPeerShardInfo();
  }

  @java.lang.Override
  public final com.google.protobuf.UnknownFieldSet
  getUnknownFields() {
    return this.unknownFields;
  }
  private TMGetPeerShardInfo(
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
          case 8: {
            bitField0_ |= 0x00000001;
            hops_ = input.readUInt32();
            break;
          }
          case 16: {
            bitField0_ |= 0x00000002;
            lastLink_ = input.readBool();
            break;
          }
          case 26: {
            if (!((mutable_bitField0_ & 0x00000004) != 0)) {
              peerChain_ = new java.util.ArrayList<protocol.TMLink>();
              mutable_bitField0_ |= 0x00000004;
            }
            peerChain_.add(
                input.readMessage(protocol.TMLink.PARSER, extensionRegistry));
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
        peerChain_ = java.util.Collections.unmodifiableList(peerChain_);
      }
      this.unknownFields = unknownFields.build();
      makeExtensionsImmutable();
    }
  }
  public static final com.google.protobuf.Descriptors.Descriptor
      getDescriptor() {
    return protocol.Ripple.internal_static_protocol_TMGetPeerShardInfo_descriptor;
  }

  @java.lang.Override
  protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internalGetFieldAccessorTable() {
    return protocol.Ripple.internal_static_protocol_TMGetPeerShardInfo_fieldAccessorTable
        .ensureFieldAccessorsInitialized(
            protocol.TMGetPeerShardInfo.class, protocol.TMGetPeerShardInfo.Builder.class);
  }

  private int bitField0_;
  public static final int HOPS_FIELD_NUMBER = 1;
  private int hops_;
  /**
   * <pre>
   * number of hops to travel
   * </pre>
   *
   * <code>required uint32 hops = 1;</code>
   * @return Whether the hops field is set.
   */
  @java.lang.Override
  public boolean hasHops() {
    return ((bitField0_ & 0x00000001) != 0);
  }
  /**
   * <pre>
   * number of hops to travel
   * </pre>
   *
   * <code>required uint32 hops = 1;</code>
   * @return The hops.
   */
  @java.lang.Override
  public int getHops() {
    return hops_;
  }

  public static final int LASTLINK_FIELD_NUMBER = 2;
  private boolean lastLink_;
  /**
   * <pre>
   * true if last link in the peer chain
   * </pre>
   *
   * <code>optional bool lastLink = 2;</code>
   * @return Whether the lastLink field is set.
   */
  @java.lang.Override
  public boolean hasLastLink() {
    return ((bitField0_ & 0x00000002) != 0);
  }
  /**
   * <pre>
   * true if last link in the peer chain
   * </pre>
   *
   * <code>optional bool lastLink = 2;</code>
   * @return The lastLink.
   */
  @java.lang.Override
  public boolean getLastLink() {
    return lastLink_;
  }

  public static final int PEERCHAIN_FIELD_NUMBER = 3;
  private java.util.List<protocol.TMLink> peerChain_;
  /**
   * <pre>
   * public keys used to route messages
   * </pre>
   *
   * <code>repeated .protocol.TMLink peerChain = 3;</code>
   */
  @java.lang.Override
  public java.util.List<protocol.TMLink> getPeerChainList() {
    return peerChain_;
  }
  /**
   * <pre>
   * public keys used to route messages
   * </pre>
   *
   * <code>repeated .protocol.TMLink peerChain = 3;</code>
   */
  @java.lang.Override
  public java.util.List<? extends protocol.TMLinkOrBuilder> 
      getPeerChainOrBuilderList() {
    return peerChain_;
  }
  /**
   * <pre>
   * public keys used to route messages
   * </pre>
   *
   * <code>repeated .protocol.TMLink peerChain = 3;</code>
   */
  @java.lang.Override
  public int getPeerChainCount() {
    return peerChain_.size();
  }
  /**
   * <pre>
   * public keys used to route messages
   * </pre>
   *
   * <code>repeated .protocol.TMLink peerChain = 3;</code>
   */
  @java.lang.Override
  public protocol.TMLink getPeerChain(int index) {
    return peerChain_.get(index);
  }
  /**
   * <pre>
   * public keys used to route messages
   * </pre>
   *
   * <code>repeated .protocol.TMLink peerChain = 3;</code>
   */
  @java.lang.Override
  public protocol.TMLinkOrBuilder getPeerChainOrBuilder(
      int index) {
    return peerChain_.get(index);
  }

  private byte memoizedIsInitialized = -1;
  @java.lang.Override
  public final boolean isInitialized() {
    byte isInitialized = memoizedIsInitialized;
    if (isInitialized == 1) return true;
    if (isInitialized == 0) return false;

    if (!hasHops()) {
      memoizedIsInitialized = 0;
      return false;
    }
    for (int i = 0; i < getPeerChainCount(); i++) {
      if (!getPeerChain(i).isInitialized()) {
        memoizedIsInitialized = 0;
        return false;
      }
    }
    memoizedIsInitialized = 1;
    return true;
  }

  @java.lang.Override
  public void writeTo(com.google.protobuf.CodedOutputStream output)
                      throws java.io.IOException {
    if (((bitField0_ & 0x00000001) != 0)) {
      output.writeUInt32(1, hops_);
    }
    if (((bitField0_ & 0x00000002) != 0)) {
      output.writeBool(2, lastLink_);
    }
    for (int i = 0; i < peerChain_.size(); i++) {
      output.writeMessage(3, peerChain_.get(i));
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
        .computeUInt32Size(1, hops_);
    }
    if (((bitField0_ & 0x00000002) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeBoolSize(2, lastLink_);
    }
    for (int i = 0; i < peerChain_.size(); i++) {
      size += com.google.protobuf.CodedOutputStream
        .computeMessageSize(3, peerChain_.get(i));
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
    if (!(obj instanceof protocol.TMGetPeerShardInfo)) {
      return super.equals(obj);
    }
    protocol.TMGetPeerShardInfo other = (protocol.TMGetPeerShardInfo) obj;

    if (hasHops() != other.hasHops()) return false;
    if (hasHops()) {
      if (getHops()
          != other.getHops()) return false;
    }
    if (hasLastLink() != other.hasLastLink()) return false;
    if (hasLastLink()) {
      if (getLastLink()
          != other.getLastLink()) return false;
    }
    if (!getPeerChainList()
        .equals(other.getPeerChainList())) return false;
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
    if (hasHops()) {
      hash = (37 * hash) + HOPS_FIELD_NUMBER;
      hash = (53 * hash) + getHops();
    }
    if (hasLastLink()) {
      hash = (37 * hash) + LASTLINK_FIELD_NUMBER;
      hash = (53 * hash) + com.google.protobuf.Internal.hashBoolean(
          getLastLink());
    }
    if (getPeerChainCount() > 0) {
      hash = (37 * hash) + PEERCHAIN_FIELD_NUMBER;
      hash = (53 * hash) + getPeerChainList().hashCode();
    }
    hash = (29 * hash) + unknownFields.hashCode();
    memoizedHashCode = hash;
    return hash;
  }

  public static protocol.TMGetPeerShardInfo parseFrom(
      java.nio.ByteBuffer data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMGetPeerShardInfo parseFrom(
      java.nio.ByteBuffer data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMGetPeerShardInfo parseFrom(
      com.google.protobuf.ByteString data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMGetPeerShardInfo parseFrom(
      com.google.protobuf.ByteString data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMGetPeerShardInfo parseFrom(byte[] data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMGetPeerShardInfo parseFrom(
      byte[] data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMGetPeerShardInfo parseFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMGetPeerShardInfo parseFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMGetPeerShardInfo parseDelimitedFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input);
  }
  public static protocol.TMGetPeerShardInfo parseDelimitedFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMGetPeerShardInfo parseFrom(
      com.google.protobuf.CodedInputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMGetPeerShardInfo parseFrom(
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
  public static Builder newBuilder(protocol.TMGetPeerShardInfo prototype) {
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
   * Request info on shards held
   * </pre>
   *
   * Protobuf type {@code protocol.TMGetPeerShardInfo}
   */
  public static final class Builder extends
      com.google.protobuf.GeneratedMessageV3.Builder<Builder> implements
      // @@protoc_insertion_point(builder_implements:protocol.TMGetPeerShardInfo)
      protocol.TMGetPeerShardInfoOrBuilder {
    public static final com.google.protobuf.Descriptors.Descriptor
        getDescriptor() {
      return protocol.Ripple.internal_static_protocol_TMGetPeerShardInfo_descriptor;
    }

    @java.lang.Override
    protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
        internalGetFieldAccessorTable() {
      return protocol.Ripple.internal_static_protocol_TMGetPeerShardInfo_fieldAccessorTable
          .ensureFieldAccessorsInitialized(
              protocol.TMGetPeerShardInfo.class, protocol.TMGetPeerShardInfo.Builder.class);
    }

    // Construct using protocol.TMGetPeerShardInfo.newBuilder()
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
        getPeerChainFieldBuilder();
      }
    }
    @java.lang.Override
    public Builder clear() {
      super.clear();
      hops_ = 0;
      bitField0_ = (bitField0_ & ~0x00000001);
      lastLink_ = false;
      bitField0_ = (bitField0_ & ~0x00000002);
      if (peerChainBuilder_ == null) {
        peerChain_ = java.util.Collections.emptyList();
        bitField0_ = (bitField0_ & ~0x00000004);
      } else {
        peerChainBuilder_.clear();
      }
      return this;
    }

    @java.lang.Override
    public com.google.protobuf.Descriptors.Descriptor
        getDescriptorForType() {
      return protocol.Ripple.internal_static_protocol_TMGetPeerShardInfo_descriptor;
    }

    @java.lang.Override
    public protocol.TMGetPeerShardInfo getDefaultInstanceForType() {
      return protocol.TMGetPeerShardInfo.getDefaultInstance();
    }

    @java.lang.Override
    public protocol.TMGetPeerShardInfo build() {
      protocol.TMGetPeerShardInfo result = buildPartial();
      if (!result.isInitialized()) {
        throw newUninitializedMessageException(result);
      }
      return result;
    }

    @java.lang.Override
    public protocol.TMGetPeerShardInfo buildPartial() {
      protocol.TMGetPeerShardInfo result = new protocol.TMGetPeerShardInfo(this);
      int from_bitField0_ = bitField0_;
      int to_bitField0_ = 0;
      if (((from_bitField0_ & 0x00000001) != 0)) {
        result.hops_ = hops_;
        to_bitField0_ |= 0x00000001;
      }
      if (((from_bitField0_ & 0x00000002) != 0)) {
        result.lastLink_ = lastLink_;
        to_bitField0_ |= 0x00000002;
      }
      if (peerChainBuilder_ == null) {
        if (((bitField0_ & 0x00000004) != 0)) {
          peerChain_ = java.util.Collections.unmodifiableList(peerChain_);
          bitField0_ = (bitField0_ & ~0x00000004);
        }
        result.peerChain_ = peerChain_;
      } else {
        result.peerChain_ = peerChainBuilder_.build();
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
      if (other instanceof protocol.TMGetPeerShardInfo) {
        return mergeFrom((protocol.TMGetPeerShardInfo)other);
      } else {
        super.mergeFrom(other);
        return this;
      }
    }

    public Builder mergeFrom(protocol.TMGetPeerShardInfo other) {
      if (other == protocol.TMGetPeerShardInfo.getDefaultInstance()) return this;
      if (other.hasHops()) {
        setHops(other.getHops());
      }
      if (other.hasLastLink()) {
        setLastLink(other.getLastLink());
      }
      if (peerChainBuilder_ == null) {
        if (!other.peerChain_.isEmpty()) {
          if (peerChain_.isEmpty()) {
            peerChain_ = other.peerChain_;
            bitField0_ = (bitField0_ & ~0x00000004);
          } else {
            ensurePeerChainIsMutable();
            peerChain_.addAll(other.peerChain_);
          }
          onChanged();
        }
      } else {
        if (!other.peerChain_.isEmpty()) {
          if (peerChainBuilder_.isEmpty()) {
            peerChainBuilder_.dispose();
            peerChainBuilder_ = null;
            peerChain_ = other.peerChain_;
            bitField0_ = (bitField0_ & ~0x00000004);
            peerChainBuilder_ = 
              com.google.protobuf.GeneratedMessageV3.alwaysUseFieldBuilders ?
                 getPeerChainFieldBuilder() : null;
          } else {
            peerChainBuilder_.addAllMessages(other.peerChain_);
          }
        }
      }
      this.mergeUnknownFields(other.unknownFields);
      onChanged();
      return this;
    }

    @java.lang.Override
    public final boolean isInitialized() {
      if (!hasHops()) {
        return false;
      }
      for (int i = 0; i < getPeerChainCount(); i++) {
        if (!getPeerChain(i).isInitialized()) {
          return false;
        }
      }
      return true;
    }

    @java.lang.Override
    public Builder mergeFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws java.io.IOException {
      protocol.TMGetPeerShardInfo parsedMessage = null;
      try {
        parsedMessage = PARSER.parsePartialFrom(input, extensionRegistry);
      } catch (com.google.protobuf.InvalidProtocolBufferException e) {
        parsedMessage = (protocol.TMGetPeerShardInfo) e.getUnfinishedMessage();
        throw e.unwrapIOException();
      } finally {
        if (parsedMessage != null) {
          mergeFrom(parsedMessage);
        }
      }
      return this;
    }
    private int bitField0_;

    private int hops_ ;
    /**
     * <pre>
     * number of hops to travel
     * </pre>
     *
     * <code>required uint32 hops = 1;</code>
     * @return Whether the hops field is set.
     */
    @java.lang.Override
    public boolean hasHops() {
      return ((bitField0_ & 0x00000001) != 0);
    }
    /**
     * <pre>
     * number of hops to travel
     * </pre>
     *
     * <code>required uint32 hops = 1;</code>
     * @return The hops.
     */
    @java.lang.Override
    public int getHops() {
      return hops_;
    }
    /**
     * <pre>
     * number of hops to travel
     * </pre>
     *
     * <code>required uint32 hops = 1;</code>
     * @param value The hops to set.
     * @return This builder for chaining.
     */
    public Builder setHops(int value) {
      bitField0_ |= 0x00000001;
      hops_ = value;
      onChanged();
      return this;
    }
    /**
     * <pre>
     * number of hops to travel
     * </pre>
     *
     * <code>required uint32 hops = 1;</code>
     * @return This builder for chaining.
     */
    public Builder clearHops() {
      bitField0_ = (bitField0_ & ~0x00000001);
      hops_ = 0;
      onChanged();
      return this;
    }

    private boolean lastLink_ ;
    /**
     * <pre>
     * true if last link in the peer chain
     * </pre>
     *
     * <code>optional bool lastLink = 2;</code>
     * @return Whether the lastLink field is set.
     */
    @java.lang.Override
    public boolean hasLastLink() {
      return ((bitField0_ & 0x00000002) != 0);
    }
    /**
     * <pre>
     * true if last link in the peer chain
     * </pre>
     *
     * <code>optional bool lastLink = 2;</code>
     * @return The lastLink.
     */
    @java.lang.Override
    public boolean getLastLink() {
      return lastLink_;
    }
    /**
     * <pre>
     * true if last link in the peer chain
     * </pre>
     *
     * <code>optional bool lastLink = 2;</code>
     * @param value The lastLink to set.
     * @return This builder for chaining.
     */
    public Builder setLastLink(boolean value) {
      bitField0_ |= 0x00000002;
      lastLink_ = value;
      onChanged();
      return this;
    }
    /**
     * <pre>
     * true if last link in the peer chain
     * </pre>
     *
     * <code>optional bool lastLink = 2;</code>
     * @return This builder for chaining.
     */
    public Builder clearLastLink() {
      bitField0_ = (bitField0_ & ~0x00000002);
      lastLink_ = false;
      onChanged();
      return this;
    }

    private java.util.List<protocol.TMLink> peerChain_ =
      java.util.Collections.emptyList();
    private void ensurePeerChainIsMutable() {
      if (!((bitField0_ & 0x00000004) != 0)) {
        peerChain_ = new java.util.ArrayList<protocol.TMLink>(peerChain_);
        bitField0_ |= 0x00000004;
       }
    }

    private com.google.protobuf.RepeatedFieldBuilderV3<
        protocol.TMLink, protocol.TMLink.Builder, protocol.TMLinkOrBuilder> peerChainBuilder_;

    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public java.util.List<protocol.TMLink> getPeerChainList() {
      if (peerChainBuilder_ == null) {
        return java.util.Collections.unmodifiableList(peerChain_);
      } else {
        return peerChainBuilder_.getMessageList();
      }
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public int getPeerChainCount() {
      if (peerChainBuilder_ == null) {
        return peerChain_.size();
      } else {
        return peerChainBuilder_.getCount();
      }
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public protocol.TMLink getPeerChain(int index) {
      if (peerChainBuilder_ == null) {
        return peerChain_.get(index);
      } else {
        return peerChainBuilder_.getMessage(index);
      }
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public Builder setPeerChain(
        int index, protocol.TMLink value) {
      if (peerChainBuilder_ == null) {
        if (value == null) {
          throw new NullPointerException();
        }
        ensurePeerChainIsMutable();
        peerChain_.set(index, value);
        onChanged();
      } else {
        peerChainBuilder_.setMessage(index, value);
      }
      return this;
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public Builder setPeerChain(
        int index, protocol.TMLink.Builder builderForValue) {
      if (peerChainBuilder_ == null) {
        ensurePeerChainIsMutable();
        peerChain_.set(index, builderForValue.build());
        onChanged();
      } else {
        peerChainBuilder_.setMessage(index, builderForValue.build());
      }
      return this;
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public Builder addPeerChain(protocol.TMLink value) {
      if (peerChainBuilder_ == null) {
        if (value == null) {
          throw new NullPointerException();
        }
        ensurePeerChainIsMutable();
        peerChain_.add(value);
        onChanged();
      } else {
        peerChainBuilder_.addMessage(value);
      }
      return this;
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public Builder addPeerChain(
        int index, protocol.TMLink value) {
      if (peerChainBuilder_ == null) {
        if (value == null) {
          throw new NullPointerException();
        }
        ensurePeerChainIsMutable();
        peerChain_.add(index, value);
        onChanged();
      } else {
        peerChainBuilder_.addMessage(index, value);
      }
      return this;
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public Builder addPeerChain(
        protocol.TMLink.Builder builderForValue) {
      if (peerChainBuilder_ == null) {
        ensurePeerChainIsMutable();
        peerChain_.add(builderForValue.build());
        onChanged();
      } else {
        peerChainBuilder_.addMessage(builderForValue.build());
      }
      return this;
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public Builder addPeerChain(
        int index, protocol.TMLink.Builder builderForValue) {
      if (peerChainBuilder_ == null) {
        ensurePeerChainIsMutable();
        peerChain_.add(index, builderForValue.build());
        onChanged();
      } else {
        peerChainBuilder_.addMessage(index, builderForValue.build());
      }
      return this;
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public Builder addAllPeerChain(
        java.lang.Iterable<? extends protocol.TMLink> values) {
      if (peerChainBuilder_ == null) {
        ensurePeerChainIsMutable();
        com.google.protobuf.AbstractMessageLite.Builder.addAll(
            values, peerChain_);
        onChanged();
      } else {
        peerChainBuilder_.addAllMessages(values);
      }
      return this;
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public Builder clearPeerChain() {
      if (peerChainBuilder_ == null) {
        peerChain_ = java.util.Collections.emptyList();
        bitField0_ = (bitField0_ & ~0x00000004);
        onChanged();
      } else {
        peerChainBuilder_.clear();
      }
      return this;
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public Builder removePeerChain(int index) {
      if (peerChainBuilder_ == null) {
        ensurePeerChainIsMutable();
        peerChain_.remove(index);
        onChanged();
      } else {
        peerChainBuilder_.remove(index);
      }
      return this;
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public protocol.TMLink.Builder getPeerChainBuilder(
        int index) {
      return getPeerChainFieldBuilder().getBuilder(index);
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public protocol.TMLinkOrBuilder getPeerChainOrBuilder(
        int index) {
      if (peerChainBuilder_ == null) {
        return peerChain_.get(index);  } else {
        return peerChainBuilder_.getMessageOrBuilder(index);
      }
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public java.util.List<? extends protocol.TMLinkOrBuilder> 
         getPeerChainOrBuilderList() {
      if (peerChainBuilder_ != null) {
        return peerChainBuilder_.getMessageOrBuilderList();
      } else {
        return java.util.Collections.unmodifiableList(peerChain_);
      }
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public protocol.TMLink.Builder addPeerChainBuilder() {
      return getPeerChainFieldBuilder().addBuilder(
          protocol.TMLink.getDefaultInstance());
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public protocol.TMLink.Builder addPeerChainBuilder(
        int index) {
      return getPeerChainFieldBuilder().addBuilder(
          index, protocol.TMLink.getDefaultInstance());
    }
    /**
     * <pre>
     * public keys used to route messages
     * </pre>
     *
     * <code>repeated .protocol.TMLink peerChain = 3;</code>
     */
    public java.util.List<protocol.TMLink.Builder> 
         getPeerChainBuilderList() {
      return getPeerChainFieldBuilder().getBuilderList();
    }
    private com.google.protobuf.RepeatedFieldBuilderV3<
        protocol.TMLink, protocol.TMLink.Builder, protocol.TMLinkOrBuilder> 
        getPeerChainFieldBuilder() {
      if (peerChainBuilder_ == null) {
        peerChainBuilder_ = new com.google.protobuf.RepeatedFieldBuilderV3<
            protocol.TMLink, protocol.TMLink.Builder, protocol.TMLinkOrBuilder>(
                peerChain_,
                ((bitField0_ & 0x00000004) != 0),
                getParentForChildren(),
                isClean());
        peerChain_ = null;
      }
      return peerChainBuilder_;
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


    // @@protoc_insertion_point(builder_scope:protocol.TMGetPeerShardInfo)
  }

  // @@protoc_insertion_point(class_scope:protocol.TMGetPeerShardInfo)
  private static final protocol.TMGetPeerShardInfo DEFAULT_INSTANCE;
  static {
    DEFAULT_INSTANCE = new protocol.TMGetPeerShardInfo();
  }

  public static protocol.TMGetPeerShardInfo getDefaultInstance() {
    return DEFAULT_INSTANCE;
  }

  @java.lang.Deprecated public static final com.google.protobuf.Parser<TMGetPeerShardInfo>
      PARSER = new com.google.protobuf.AbstractParser<TMGetPeerShardInfo>() {
    @java.lang.Override
    public TMGetPeerShardInfo parsePartialFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws com.google.protobuf.InvalidProtocolBufferException {
      return new TMGetPeerShardInfo(input, extensionRegistry);
    }
  };

  public static com.google.protobuf.Parser<TMGetPeerShardInfo> parser() {
    return PARSER;
  }

  @java.lang.Override
  public com.google.protobuf.Parser<TMGetPeerShardInfo> getParserForType() {
    return PARSER;
  }

  @java.lang.Override
  public protocol.TMGetPeerShardInfo getDefaultInstanceForType() {
    return DEFAULT_INSTANCE;
  }

}
