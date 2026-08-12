AutoconfConfigInfo = provider(
    doc = "Ordered autoconf-style check results for generated configuration headers.",
    fields = {
        "results": "list[struct]: ordered compile, link, and policy results",
    },
)
