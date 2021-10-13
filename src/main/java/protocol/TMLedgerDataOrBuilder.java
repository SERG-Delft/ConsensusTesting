// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

public interface TMLedgerDataOrBuilder extends
    // @@protoc_insertion_point(interface_extends:protocol.TMLedgerData)
    com.google.protobuf.MessageOrBuilder {

  /**
   * <code>required bytes ledgerHash = 1;</code>
   * @return Whether the ledgerHash field is set.
   */
  boolean hasLedgerHash();
  /**
   * <code>required bytes ledgerHash = 1;</code>
   * @return The ledgerHash.
   */
  com.google.protobuf.ByteString getLedgerHash();

  /**
   * <code>required uint32 ledgerSeq = 2;</code>
   * @return Whether the ledgerSeq field is set.
   */
  boolean hasLedgerSeq();
  /**
   * <code>required uint32 ledgerSeq = 2;</code>
   * @return The ledgerSeq.
   */
  int getLedgerSeq();

  /**
   * <code>required .protocol.TMLedgerInfoType type = 3;</code>
   * @return Whether the type field is set.
   */
  boolean hasType();
  /**
   * <code>required .protocol.TMLedgerInfoType type = 3;</code>
   * @return The type.
   */
  protocol.TMLedgerInfoType getType();

  /**
   * <code>repeated .protocol.TMLedgerNode nodes = 4;</code>
   */
  java.util.List<protocol.TMLedgerNode> 
      getNodesList();
  /**
   * <code>repeated .protocol.TMLedgerNode nodes = 4;</code>
   */
  protocol.TMLedgerNode getNodes(int index);
  /**
   * <code>repeated .protocol.TMLedgerNode nodes = 4;</code>
   */
  int getNodesCount();
  /**
   * <code>repeated .protocol.TMLedgerNode nodes = 4;</code>
   */
  java.util.List<? extends protocol.TMLedgerNodeOrBuilder> 
      getNodesOrBuilderList();
  /**
   * <code>repeated .protocol.TMLedgerNode nodes = 4;</code>
   */
  protocol.TMLedgerNodeOrBuilder getNodesOrBuilder(
      int index);

  /**
   * <code>optional uint32 requestCookie = 5;</code>
   * @return Whether the requestCookie field is set.
   */
  boolean hasRequestCookie();
  /**
   * <code>optional uint32 requestCookie = 5;</code>
   * @return The requestCookie.
   */
  int getRequestCookie();

  /**
   * <code>optional .protocol.TMReplyError error = 6;</code>
   * @return Whether the error field is set.
   */
  boolean hasError();
  /**
   * <code>optional .protocol.TMReplyError error = 6;</code>
   * @return The error.
   */
  protocol.TMReplyError getError();
}