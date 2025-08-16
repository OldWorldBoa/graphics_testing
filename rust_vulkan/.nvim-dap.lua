local dap = require("dap")

dap.adapters.lldb = {
	type = "executable",
	command = "/usr/bin/rust-lldb",
	name = "lldb",
}

dap.configurations.rust = {
	{
		name = "rust_vulkan",
		type = "lldb",
		request = "launch",
		program = function()
			return vim.fn.getcwd() .. "./target/debug/rust_vulkan"
		end,
		cwd = "${workspaceFolder}",
		stopOnEntry = false,
	},
}
