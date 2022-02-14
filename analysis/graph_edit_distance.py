import networkx as nx
import pygraphviz

def parse_dot_file(file_name):
    with open(file_name, 'r') as fp:
        return nx.drawing.nx_agraph.read_dot(fp)


def calculate_ged(g1, g2):
    return nx.algorithms.similarity.graph_edit_distance(g1, g2)


if __name__ == '__main__':
    graph1 = parse_dot_file('graph1.dot')
    graph2 = parse_dot_file('graph2.dot')
    print(graph1)
    print(graph2)
    print(calculate_ged(graph1, graph2))

