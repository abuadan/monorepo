"""
Pipeline Graph definition
"""

from typing import Any, Iterable, Mapping, NewType

import networkx as nx

from dataplatform.compute.base import ComputeProtocol

UniqueId = NewType("UniqueId", str)


class Documentation(str):
    pass


class Graph:
    """ """

    def __init__(
        self,
        name: str,
        start_compute_step: ComputeProtocol,
        end_compute_step: ComputeProtocol,
        dependencies: set["Graph"],
        documentation: Documentation,
        metadata: Mapping[Any, Any],
    ) -> None:
        self.name = name
        self.metadata = metadata
        self.documentation = documentation
        self.dependencies = dependencies
        self.start = start_compute_step
        self.end = end_compute_step
        self.graph: nx.DiGraph = self.construct_graph(
            start_compute_step, end_compute_step
        )
        self.__validate_graph()

    @staticmethod
    def construct_graph(
        start_compute_step: ComputeProtocol, end_compute_step: ComputeProtocol
    ) -> nx.DiGraph:
        """ """

        dependency_graph = end_compute_step.adjacency_matrix()
        for _, dependency in end_compute_step.dependencies.items():
            if dependency.name != start_compute_step.name:
                adjecency = dependency.adjacency_matrix()
                dependency_graph.extend(adjecency)
            else:
                break

        G = nx.DiGraph(dependency_graph)
        for layer, nodes in enumerate(nx.topological_generations(G)):  # pyright: ignore
            # `multipartite_layout` expects the layer as a node attribute, so add the
            # numeric layer value as a node attribute
            for node in nodes:
                G.nodes[node]["layer"] = layer
        return G

    def __validate_graph(self):
        """
        Validate that the constructed Graph
        does not contain any cycles, and the
        graph was constructed correctly.
        """
        cycles = list(nx.simple_cycles(self.graph))
        if cycles:
            raise ValueError(f"No cycles allowed: {cycles}")

    def subgraph(self, nodes: Iterable[UniqueId]) -> nx.Graph:
        # Take the original networkx graph and return a subgraph containing only
        # the selected unique_id nodes.
        return self.graph.subgraph(nodes)
