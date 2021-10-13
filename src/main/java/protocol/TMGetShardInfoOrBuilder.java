// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

public interface TMGetShardInfoOrBuilder extends
    // @@protoc_insertion_point(interface_extends:protocol.TMGetShardInfo)
    com.google.protobuf.MessageOrBuilder {

  /**
   * <pre>
   * number of hops to travel
   * </pre>
   *
   * <code>required uint32 hops = 1 [deprecated = true];</code>
   * @return Whether the hops field is set.
   */
  @java.lang.Deprecated boolean hasHops();
  /**
   * <pre>
   * number of hops to travel
   * </pre>
   *
   * <code>required uint32 hops = 1 [deprecated = true];</code>
   * @return The hops.
   */
  @java.lang.Deprecated int getHops();

  /**
   * <pre>
   * true if last link in the peer chain
   * </pre>
   *
   * <code>optional bool lastLink = 2 [deprecated = true];</code>
   * @return Whether the lastLink field is set.
   */
  @java.lang.Deprecated boolean hasLastLink();
  /**
   * <pre>
   * true if last link in the peer chain
   * </pre>
   *
   * <code>optional bool lastLink = 2 [deprecated = true];</code>
   * @return The lastLink.
   */
  @java.lang.Deprecated boolean getLastLink();

  /**
   * <pre>
   * IDs used to route messages
   * </pre>
   *
   * <code>repeated uint32 peerchain = 3 [deprecated = true];</code>
   * @return A list containing the peerchain.
   */
  @java.lang.Deprecated java.util.List<java.lang.Integer> getPeerchainList();
  /**
   * <pre>
   * IDs used to route messages
   * </pre>
   *
   * <code>repeated uint32 peerchain = 3 [deprecated = true];</code>
   * @return The count of peerchain.
   */
  @java.lang.Deprecated int getPeerchainCount();
  /**
   * <pre>
   * IDs used to route messages
   * </pre>
   *
   * <code>repeated uint32 peerchain = 3 [deprecated = true];</code>
   * @param index The index of the element to return.
   * @return The peerchain at the given index.
   */
  @java.lang.Deprecated int getPeerchain(int index);
}