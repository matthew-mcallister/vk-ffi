gen:
	PYTHONPATH="vendor/vkreg-py" bin/bindgen -i vendor/Vulkan-Docs/xml/vk.xml -o generated

test:
	cargo test --all-features
