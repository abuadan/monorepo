import networkx as nx
import pygraphviz


def test_pygraphviz():
    G = nx.complete_graph(5)
    A = nx.nx_agraph.to_agraph(G)
    print("PyGraphviz version:", pygraphviz.__version__)
    print("Graph string:", A.string())


if __name__ == "__main__":
    test_pygraphviz()
