// Generated by the protocol buffer compiler.  DO NOT EDIT!
// source: proto/ripple.proto

package protocol;

public interface TMGetPeerShardInfoOrBuilder extends
    // @@protoc_insertion_point(interface_extends:protocol.TMGetPeerShardInfo)
    com.google.protobuf.MessageOrBuilder {

  /**
   * <pre>
   * number of hops to travel
   * </pre>
   *
   * <code>required uint32 hops = 1;</code>
   * @return Whether the hops field is set.
   */
  boolean hasHops();
  /**
   * <pre>
   * number of hops to travel
   * </pre>
   *
   * <code>required uint32 hops = 1;</code>
   * @return The hops.
   */
  int getHops();

  /**
   * <pre>
   * true if last link in the peer chain
   * </pre>
   *
   * <code>optional bool lastLink = 2;</code>
   * @return Whether the lastLink field is set.
   */
  boolean hasLastLink();
  /**
   * <pre>
   * true if last link in the peer chain
   * </pre>
   *
   * <code>optional bool lastLink = 2;</code>
   * @return The lastLink.
   */
  boolean getLastLink();

  /**
   * <pre>
   * public keys used to route messages
   * </pre>
   *
   * <code>repeated .protocol.TMLink peerChain = 3;</code>
   */
  java.util.List<protocol.TMLink> 
      getPeerChainList();
  /**
   * <pre>
   * public keys used to route messages
   * </pre>
   *
   * <code>repeated .protocol.TMLink peerChain = 3;</code>
   */
  protocol.TMLink getPeerChain(int index);
  /**
   * <pre>
   * public keys used to route messages
   * </pre>
   *
   * <code>repeated .protocol.TMLink peerChain = 3;</code>
   */
  int getPeerChainCount();
  /**
   * <pre>
   * public keys used to route messages
   * </pre>
   *
   * <code>repeated .protocol.TMLink peerChain = 3;</code>
   */
  java.util.List<? extends protocol.TMLinkOrBuilder> 
      getPeerChainOrBuilderList();
  /**
   * <pre>
   * public keys used to route messages
   * </pre>
   *
   * <code>repeated .protocol.TMLink peerChain = 3;</code>
   */
  protocol.TMLinkOrBuilder getPeerChainOrBuilder(
      int index);
}