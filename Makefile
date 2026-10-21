.PHONY: build run test check gdb trace doctor

build:
	bash scripts/build.sh $(ARGS)

run:
	bash scripts/qemu-run.sh $(ARGS)

trace:
	bash scripts/qemu-trace.sh $(ARGS)

gdb:
	bash scripts/qemu-gdb.sh $(ARGS)

test:
	bash scripts/test.sh $(ARGS)

check:
	bash scripts/check.sh $(ARGS)

doctor:
	bash scripts/doctor.sh $(ARGS)
