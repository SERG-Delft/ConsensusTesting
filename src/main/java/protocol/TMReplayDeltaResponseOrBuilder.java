// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

public interface TMReplayDeltaResponseOrBuilder extends
    // @@protoc_insertion_point(interface_extends:protocol.TMReplayDeltaResponse)
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
   * <code>optional bytes ledgerHeader = 2;</code>
   * @return Whether the ledgerHeader field is set.
   */
  boolean hasLedgerHeader();
  /**
   * <code>optional bytes ledgerHeader = 2;</code>
   * @return The ledgerHeader.
   */
  com.google.protobuf.ByteString getLedgerHeader();

  /**
   * <code>repeated bytes transaction = 3;</code>
   * @return A list containing the transaction.
   */
  java.util.List<com.google.protobuf.ByteString> getTransactionList();
  /**
   * <code>repeated bytes transaction = 3;</code>
   * @return The count of transaction.
   */
  int getTransactionCount();
  /**
   * <code>repeated bytes transaction = 3;</code>
   * @param index The index of the element to return.
   * @return The transaction at the given index.
   */
  com.google.protobuf.ByteString getTransaction(int index);

  /**
   * <code>optional .protocol.TMReplyError error = 4;</code>
   * @return Whether the error field is set.
   */
  boolean hasError();
  /**
   * <code>optional .protocol.TMReplyError error = 4;</code>
   * @return The error.
   */
  protocol.TMReplyError getError();
}
