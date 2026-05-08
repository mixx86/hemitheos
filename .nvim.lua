return {
	{
		"neovim/nvim-lspconfig",
		opts = {
			servers = {
				rust_analyzer = {
					settings = {
						["rust-analyzer"] = {
							cargo = {
								target = "aarch64-unknown-none",
								diagnostics = {
									disabled = { "inactive-code" },
								},
							},
						},
					},
				},
			},
		},
	},
}
