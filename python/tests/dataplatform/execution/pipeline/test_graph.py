"""
Test Graph construction
"""

from typing import Mapping, Optional
from unittest.mock import MagicMock, Mock

import pytest

from dataplatform.compute import BaseCompute, Compute_T, ComputeProtocol
from dataplatform.compute.schema import SchemaProtocol
from dataplatform.execution.pipeline import Graph
from dataplatform.execution.pipeline.graph import Documentation
from dataplatform.storage.protocol import SupportsStorage


class TestCompute(BaseCompute):
    def __init__(
        self,
        name: str,
        sink: SupportsStorage,
        dependencies: Mapping[str, ComputeProtocol],
        schema: SchemaProtocol,
        compute_type: str,
        config: Optional[Compute_T] = None,
    ) -> None:
        super().__init__(name, sink, dependencies, schema, compute_type, config)

    @property
    def input_data(self):
        return None

    def execute(self, function=None) -> None:
        return None


@pytest.fixture(name="long_compute_chain")
def long_compute_chain_fixutre():
    start = TestCompute(
        name="start",
        sink=MagicMock(spec=SupportsStorage),
        compute_type="test",
        dependencies={"before_start": MagicMock(spec=ComputeProtocol)},
        schema=Mock(),
    )

    after_start = TestCompute(
        name="after_start",
        sink=MagicMock(spec=SupportsStorage),
        compute_type="test",
        dependencies={"start": start},
        schema=Mock(),
    )

    end = TestCompute(
        name="end",
        sink=MagicMock(spec=SupportsStorage),
        compute_type="test",
        dependencies={"after_start": after_start, "start": start},
        schema=Mock(),
    )

    return [start, after_start, end]


def test_graph_generation(long_compute_chain):
    """"""
    start, *_, end = long_compute_chain
    graph = Graph(
        name="test_graph",
        start_compute_step=start,
        end_compute_step=end,
        dependencies=set(),
        documentation=Documentation("test docs."),
        metadata={},
    )

    breakpoint()
    assert graph.graph is None
