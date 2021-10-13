// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

public interface TMProofPathRequestOrBuilder extends
    // @@protoc_insertion_point(interface_extends:protocol.TMProofPathRequest)
    com.google.protobuf.MessageOrBuilder {

  /**
   * <code>required bytes key = 1;</code>
   * @return Whether the key field is set.
   */
  boolean hasKey();
  /**
   * <code>required bytes key = 1;</code>
   * @return The key.
   */
  com.google.protobuf.ByteString getKey();

  /**
   * <code>required bytes ledgerHash = 2;</code>
   * @return Whether the ledgerHash field is set.
   */
  boolean hasLedgerHash();
  /**
   * <code>required bytes ledgerHash = 2;</code>
   * @return The ledgerHash.
   */
  com.google.protobuf.ByteString getLedgerHash();

  /**
   * <code>required .protocol.TMLedgerMapType type = 3;</code>
   * @return Whether the type field is set.
   */
  boolean hasType();
  /**
   * <code>required .protocol.TMLedgerMapType type = 3;</code>
   * @return The type.
   */
  protocol.TMLedgerMapType getType();
}