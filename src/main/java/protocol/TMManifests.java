// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

/**
 * Protobuf type {@code protocol.TMManifests}
 */
public final class TMManifests extends
    com.google.protobuf.GeneratedMessageV3 implements
    // @@protoc_insertion_point(message_implements:protocol.TMManifests)
    TMManifestsOrBuilder {
private static final long serialVersionUID = 0L;
  // Use TMManifests.newBuilder() to construct.
  private TMManifests(com.google.protobuf.GeneratedMessageV3.Builder<?> builder) {
    super(builder);
  }
  private TMManifests() {
    list_ = java.util.Collections.emptyList();
  }

  @java.lang.Override
  @SuppressWarnings({"unused"})
  protected java.lang.Object newInstance(
      UnusedPrivateParameter unused) {
    return new TMManifests();
  }

  @java.lang.Override
  public final com.google.protobuf.UnknownFieldSet
  getUnknownFields() {
    return this.unknownFields;
  }
  private TMManifests(
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
            if (!((mutable_bitField0_ & 0x00000001) != 0)) {
              list_ = new java.util.ArrayList<protocol.TMManifest>();
              mutable_bitField0_ |= 0x00000001;
            }
            list_.add(
                input.readMessage(protocol.TMManifest.PARSER, extensionRegistry));
            break;
          }
          case 16: {
            bitField0_ |= 0x00000001;
            history_ = input.readBool();
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
      if (((mutable_bitField0_ & 0x00000001) != 0)) {
        list_ = java.util.Collections.unmodifiableList(list_);
      }
      this.unknownFields = unknownFields.build();
      makeExtensionsImmutable();
    }
  }
  public static final com.google.protobuf.Descriptors.Descriptor
      getDescriptor() {
    return protocol.Ripple.internal_static_protocol_TMManifests_descriptor;
  }

  @java.lang.Override
  protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
      internalGetFieldAccessorTable() {
    return protocol.Ripple.internal_static_protocol_TMManifests_fieldAccessorTable
        .ensureFieldAccessorsInitialized(
            protocol.TMManifests.class, protocol.TMManifests.Builder.class);
  }

  private int bitField0_;
  public static final int LIST_FIELD_NUMBER = 1;
  private java.util.List<protocol.TMManifest> list_;
  /**
   * <code>repeated .protocol.TMManifest list = 1;</code>
   */
  @java.lang.Override
  public java.util.List<protocol.TMManifest> getListList() {
    return list_;
  }
  /**
   * <code>repeated .protocol.TMManifest list = 1;</code>
   */
  @java.lang.Override
  public java.util.List<? extends protocol.TMManifestOrBuilder> 
      getListOrBuilderList() {
    return list_;
  }
  /**
   * <code>repeated .protocol.TMManifest list = 1;</code>
   */
  @java.lang.Override
  public int getListCount() {
    return list_.size();
  }
  /**
   * <code>repeated .protocol.TMManifest list = 1;</code>
   */
  @java.lang.Override
  public protocol.TMManifest getList(int index) {
    return list_.get(index);
  }
  /**
   * <code>repeated .protocol.TMManifest list = 1;</code>
   */
  @java.lang.Override
  public protocol.TMManifestOrBuilder getListOrBuilder(
      int index) {
    return list_.get(index);
  }

  public static final int HISTORY_FIELD_NUMBER = 2;
  private boolean history_;
  /**
   * <pre>
   * The manifests sent when a peer first connects to another peer are `history`.
   * </pre>
   *
   * <code>optional bool history = 2 [deprecated = true];</code>
   * @return Whether the history field is set.
   */
  @java.lang.Override
  @java.lang.Deprecated public boolean hasHistory() {
    return ((bitField0_ & 0x00000001) != 0);
  }
  /**
   * <pre>
   * The manifests sent when a peer first connects to another peer are `history`.
   * </pre>
   *
   * <code>optional bool history = 2 [deprecated = true];</code>
   * @return The history.
   */
  @java.lang.Override
  @java.lang.Deprecated public boolean getHistory() {
    return history_;
  }

  private byte memoizedIsInitialized = -1;
  @java.lang.Override
  public final boolean isInitialized() {
    byte isInitialized = memoizedIsInitialized;
    if (isInitialized == 1) return true;
    if (isInitialized == 0) return false;

    for (int i = 0; i < getListCount(); i++) {
      if (!getList(i).isInitialized()) {
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
    for (int i = 0; i < list_.size(); i++) {
      output.writeMessage(1, list_.get(i));
    }
    if (((bitField0_ & 0x00000001) != 0)) {
      output.writeBool(2, history_);
    }
    unknownFields.writeTo(output);
  }

  @java.lang.Override
  public int getSerializedSize() {
    int size = memoizedSize;
    if (size != -1) return size;

    size = 0;
    for (int i = 0; i < list_.size(); i++) {
      size += com.google.protobuf.CodedOutputStream
        .computeMessageSize(1, list_.get(i));
    }
    if (((bitField0_ & 0x00000001) != 0)) {
      size += com.google.protobuf.CodedOutputStream
        .computeBoolSize(2, history_);
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
    if (!(obj instanceof protocol.TMManifests)) {
      return super.equals(obj);
    }
    protocol.TMManifests other = (protocol.TMManifests) obj;

    if (!getListList()
        .equals(other.getListList())) return false;
    if (hasHistory() != other.hasHistory()) return false;
    if (hasHistory()) {
      if (getHistory()
          != other.getHistory()) return false;
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
    if (getListCount() > 0) {
      hash = (37 * hash) + LIST_FIELD_NUMBER;
      hash = (53 * hash) + getListList().hashCode();
    }
    if (hasHistory()) {
      hash = (37 * hash) + HISTORY_FIELD_NUMBER;
      hash = (53 * hash) + com.google.protobuf.Internal.hashBoolean(
          getHistory());
    }
    hash = (29 * hash) + unknownFields.hashCode();
    memoizedHashCode = hash;
    return hash;
  }

  public static protocol.TMManifests parseFrom(
      java.nio.ByteBuffer data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMManifests parseFrom(
      java.nio.ByteBuffer data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMManifests parseFrom(
      com.google.protobuf.ByteString data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMManifests parseFrom(
      com.google.protobuf.ByteString data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMManifests parseFrom(byte[] data)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data);
  }
  public static protocol.TMManifests parseFrom(
      byte[] data,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws com.google.protobuf.InvalidProtocolBufferException {
    return PARSER.parseFrom(data, extensionRegistry);
  }
  public static protocol.TMManifests parseFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMManifests parseFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMManifests parseDelimitedFrom(java.io.InputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input);
  }
  public static protocol.TMManifests parseDelimitedFrom(
      java.io.InputStream input,
      com.google.protobuf.ExtensionRegistryLite extensionRegistry)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseDelimitedWithIOException(PARSER, input, extensionRegistry);
  }
  public static protocol.TMManifests parseFrom(
      com.google.protobuf.CodedInputStream input)
      throws java.io.IOException {
    return com.google.protobuf.GeneratedMessageV3
        .parseWithIOException(PARSER, input);
  }
  public static protocol.TMManifests parseFrom(
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
  public static Builder newBuilder(protocol.TMManifests prototype) {
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
   * Protobuf type {@code protocol.TMManifests}
   */
  public static final class Builder extends
      com.google.protobuf.GeneratedMessageV3.Builder<Builder> implements
      // @@protoc_insertion_point(builder_implements:protocol.TMManifests)
      protocol.TMManifestsOrBuilder {
    public static final com.google.protobuf.Descriptors.Descriptor
        getDescriptor() {
      return protocol.Ripple.internal_static_protocol_TMManifests_descriptor;
    }

    @java.lang.Override
    protected com.google.protobuf.GeneratedMessageV3.FieldAccessorTable
        internalGetFieldAccessorTable() {
      return protocol.Ripple.internal_static_protocol_TMManifests_fieldAccessorTable
          .ensureFieldAccessorsInitialized(
              protocol.TMManifests.class, protocol.TMManifests.Builder.class);
    }

    // Construct using protocol.TMManifests.newBuilder()
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
        getListFieldBuilder();
      }
    }
    @java.lang.Override
    public Builder clear() {
      super.clear();
      if (listBuilder_ == null) {
        list_ = java.util.Collections.emptyList();
        bitField0_ = (bitField0_ & ~0x00000001);
      } else {
        listBuilder_.clear();
      }
      history_ = false;
      bitField0_ = (bitField0_ & ~0x00000002);
      return this;
    }

    @java.lang.Override
    public com.google.protobuf.Descriptors.Descriptor
        getDescriptorForType() {
      return protocol.Ripple.internal_static_protocol_TMManifests_descriptor;
    }

    @java.lang.Override
    public protocol.TMManifests getDefaultInstanceForType() {
      return protocol.TMManifests.getDefaultInstance();
    }

    @java.lang.Override
    public protocol.TMManifests build() {
      protocol.TMManifests result = buildPartial();
      if (!result.isInitialized()) {
        throw newUninitializedMessageException(result);
      }
      return result;
    }

    @java.lang.Override
    public protocol.TMManifests buildPartial() {
      protocol.TMManifests result = new protocol.TMManifests(this);
      int from_bitField0_ = bitField0_;
      int to_bitField0_ = 0;
      if (listBuilder_ == null) {
        if (((bitField0_ & 0x00000001) != 0)) {
          list_ = java.util.Collections.unmodifiableList(list_);
          bitField0_ = (bitField0_ & ~0x00000001);
        }
        result.list_ = list_;
      } else {
        result.list_ = listBuilder_.build();
      }
      if (((from_bitField0_ & 0x00000002) != 0)) {
        result.history_ = history_;
        to_bitField0_ |= 0x00000001;
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
      if (other instanceof protocol.TMManifests) {
        return mergeFrom((protocol.TMManifests)other);
      } else {
        super.mergeFrom(other);
        return this;
      }
    }

    public Builder mergeFrom(protocol.TMManifests other) {
      if (other == protocol.TMManifests.getDefaultInstance()) return this;
      if (listBuilder_ == null) {
        if (!other.list_.isEmpty()) {
          if (list_.isEmpty()) {
            list_ = other.list_;
            bitField0_ = (bitField0_ & ~0x00000001);
          } else {
            ensureListIsMutable();
            list_.addAll(other.list_);
          }
          onChanged();
        }
      } else {
        if (!other.list_.isEmpty()) {
          if (listBuilder_.isEmpty()) {
            listBuilder_.dispose();
            listBuilder_ = null;
            list_ = other.list_;
            bitField0_ = (bitField0_ & ~0x00000001);
            listBuilder_ = 
              com.google.protobuf.GeneratedMessageV3.alwaysUseFieldBuilders ?
                 getListFieldBuilder() : null;
          } else {
            listBuilder_.addAllMessages(other.list_);
          }
        }
      }
      if (other.hasHistory()) {
        setHistory(other.getHistory());
      }
      this.mergeUnknownFields(other.unknownFields);
      onChanged();
      return this;
    }

    @java.lang.Override
    public final boolean isInitialized() {
      for (int i = 0; i < getListCount(); i++) {
        if (!getList(i).isInitialized()) {
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
      protocol.TMManifests parsedMessage = null;
      try {
        parsedMessage = PARSER.parsePartialFrom(input, extensionRegistry);
      } catch (com.google.protobuf.InvalidProtocolBufferException e) {
        parsedMessage = (protocol.TMManifests) e.getUnfinishedMessage();
        throw e.unwrapIOException();
      } finally {
        if (parsedMessage != null) {
          mergeFrom(parsedMessage);
        }
      }
      return this;
    }
    private int bitField0_;

    private java.util.List<protocol.TMManifest> list_ =
      java.util.Collections.emptyList();
    private void ensureListIsMutable() {
      if (!((bitField0_ & 0x00000001) != 0)) {
        list_ = new java.util.ArrayList<protocol.TMManifest>(list_);
        bitField0_ |= 0x00000001;
       }
    }

    private com.google.protobuf.RepeatedFieldBuilderV3<
        protocol.TMManifest, protocol.TMManifest.Builder, protocol.TMManifestOrBuilder> listBuilder_;

    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public java.util.List<protocol.TMManifest> getListList() {
      if (listBuilder_ == null) {
        return java.util.Collections.unmodifiableList(list_);
      } else {
        return listBuilder_.getMessageList();
      }
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public int getListCount() {
      if (listBuilder_ == null) {
        return list_.size();
      } else {
        return listBuilder_.getCount();
      }
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public protocol.TMManifest getList(int index) {
      if (listBuilder_ == null) {
        return list_.get(index);
      } else {
        return listBuilder_.getMessage(index);
      }
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public Builder setList(
        int index, protocol.TMManifest value) {
      if (listBuilder_ == null) {
        if (value == null) {
          throw new NullPointerException();
        }
        ensureListIsMutable();
        list_.set(index, value);
        onChanged();
      } else {
        listBuilder_.setMessage(index, value);
      }
      return this;
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public Builder setList(
        int index, protocol.TMManifest.Builder builderForValue) {
      if (listBuilder_ == null) {
        ensureListIsMutable();
        list_.set(index, builderForValue.build());
        onChanged();
      } else {
        listBuilder_.setMessage(index, builderForValue.build());
      }
      return this;
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public Builder addList(protocol.TMManifest value) {
      if (listBuilder_ == null) {
        if (value == null) {
          throw new NullPointerException();
        }
        ensureListIsMutable();
        list_.add(value);
        onChanged();
      } else {
        listBuilder_.addMessage(value);
      }
      return this;
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public Builder addList(
        int index, protocol.TMManifest value) {
      if (listBuilder_ == null) {
        if (value == null) {
          throw new NullPointerException();
        }
        ensureListIsMutable();
        list_.add(index, value);
        onChanged();
      } else {
        listBuilder_.addMessage(index, value);
      }
      return this;
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public Builder addList(
        protocol.TMManifest.Builder builderForValue) {
      if (listBuilder_ == null) {
        ensureListIsMutable();
        list_.add(builderForValue.build());
        onChanged();
      } else {
        listBuilder_.addMessage(builderForValue.build());
      }
      return this;
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public Builder addList(
        int index, protocol.TMManifest.Builder builderForValue) {
      if (listBuilder_ == null) {
        ensureListIsMutable();
        list_.add(index, builderForValue.build());
        onChanged();
      } else {
        listBuilder_.addMessage(index, builderForValue.build());
      }
      return this;
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public Builder addAllList(
        java.lang.Iterable<? extends protocol.TMManifest> values) {
      if (listBuilder_ == null) {
        ensureListIsMutable();
        com.google.protobuf.AbstractMessageLite.Builder.addAll(
            values, list_);
        onChanged();
      } else {
        listBuilder_.addAllMessages(values);
      }
      return this;
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public Builder clearList() {
      if (listBuilder_ == null) {
        list_ = java.util.Collections.emptyList();
        bitField0_ = (bitField0_ & ~0x00000001);
        onChanged();
      } else {
        listBuilder_.clear();
      }
      return this;
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public Builder removeList(int index) {
      if (listBuilder_ == null) {
        ensureListIsMutable();
        list_.remove(index);
        onChanged();
      } else {
        listBuilder_.remove(index);
      }
      return this;
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public protocol.TMManifest.Builder getListBuilder(
        int index) {
      return getListFieldBuilder().getBuilder(index);
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public protocol.TMManifestOrBuilder getListOrBuilder(
        int index) {
      if (listBuilder_ == null) {
        return list_.get(index);  } else {
        return listBuilder_.getMessageOrBuilder(index);
      }
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public java.util.List<? extends protocol.TMManifestOrBuilder> 
         getListOrBuilderList() {
      if (listBuilder_ != null) {
        return listBuilder_.getMessageOrBuilderList();
      } else {
        return java.util.Collections.unmodifiableList(list_);
      }
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public protocol.TMManifest.Builder addListBuilder() {
      return getListFieldBuilder().addBuilder(
          protocol.TMManifest.getDefaultInstance());
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public protocol.TMManifest.Builder addListBuilder(
        int index) {
      return getListFieldBuilder().addBuilder(
          index, protocol.TMManifest.getDefaultInstance());
    }
    /**
     * <code>repeated .protocol.TMManifest list = 1;</code>
     */
    public java.util.List<protocol.TMManifest.Builder> 
         getListBuilderList() {
      return getListFieldBuilder().getBuilderList();
    }
    private com.google.protobuf.RepeatedFieldBuilderV3<
        protocol.TMManifest, protocol.TMManifest.Builder, protocol.TMManifestOrBuilder> 
        getListFieldBuilder() {
      if (listBuilder_ == null) {
        listBuilder_ = new com.google.protobuf.RepeatedFieldBuilderV3<
            protocol.TMManifest, protocol.TMManifest.Builder, protocol.TMManifestOrBuilder>(
                list_,
                ((bitField0_ & 0x00000001) != 0),
                getParentForChildren(),
                isClean());
        list_ = null;
      }
      return listBuilder_;
    }

    private boolean history_ ;
    /**
     * <pre>
     * The manifests sent when a peer first connects to another peer are `history`.
     * </pre>
     *
     * <code>optional bool history = 2 [deprecated = true];</code>
     * @return Whether the history field is set.
     */
    @java.lang.Override
    @java.lang.Deprecated public boolean hasHistory() {
      return ((bitField0_ & 0x00000002) != 0);
    }
    /**
     * <pre>
     * The manifests sent when a peer first connects to another peer are `history`.
     * </pre>
     *
     * <code>optional bool history = 2 [deprecated = true];</code>
     * @return The history.
     */
    @java.lang.Override
    @java.lang.Deprecated public boolean getHistory() {
      return history_;
    }
    /**
     * <pre>
     * The manifests sent when a peer first connects to another peer are `history`.
     * </pre>
     *
     * <code>optional bool history = 2 [deprecated = true];</code>
     * @param value The history to set.
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder setHistory(boolean value) {
      bitField0_ |= 0x00000002;
      history_ = value;
      onChanged();
      return this;
    }
    /**
     * <pre>
     * The manifests sent when a peer first connects to another peer are `history`.
     * </pre>
     *
     * <code>optional bool history = 2 [deprecated = true];</code>
     * @return This builder for chaining.
     */
    @java.lang.Deprecated public Builder clearHistory() {
      bitField0_ = (bitField0_ & ~0x00000002);
      history_ = false;
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


    // @@protoc_insertion_point(builder_scope:protocol.TMManifests)
  }

  // @@protoc_insertion_point(class_scope:protocol.TMManifests)
  private static final protocol.TMManifests DEFAULT_INSTANCE;
  static {
    DEFAULT_INSTANCE = new protocol.TMManifests();
  }

  public static protocol.TMManifests getDefaultInstance() {
    return DEFAULT_INSTANCE;
  }

  @java.lang.Deprecated public static final com.google.protobuf.Parser<TMManifests>
      PARSER = new com.google.protobuf.AbstractParser<TMManifests>() {
    @java.lang.Override
    public TMManifests parsePartialFrom(
        com.google.protobuf.CodedInputStream input,
        com.google.protobuf.ExtensionRegistryLite extensionRegistry)
        throws com.google.protobuf.InvalidProtocolBufferException {
      return new TMManifests(input, extensionRegistry);
    }
  };

  public static com.google.protobuf.Parser<TMManifests> parser() {
    return PARSER;
  }

  @java.lang.Override
  public com.google.protobuf.Parser<TMManifests> getParserForType() {
    return PARSER;
  }

  @java.lang.Override
  public protocol.TMManifests getDefaultInstanceForType() {
    return DEFAULT_INSTANCE;
  }

}
