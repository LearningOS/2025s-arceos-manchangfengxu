# Utility definitions and functions

GREEN_C := \033[92;1m
CYAN_C := \033[96;1m
YELLOW_C := \033[93;1m
GRAY_C := \033[90m
WHITE_C := \033[37m
END_C := \033[0m

define run_cmd
  @printf '$(WHITE_C)$(1)$(END_C) $(GRAY_C)$(2)$(END_C)\n'
  @$(1) $(2)
endef

define make_disk_image_fat32
  @printf "    $(GREEN_C)Creating$(END_C) FAT32 disk image \"$(1)\" ...\n"
  @dd if=/dev/zero of=$(1) bs=1M count=64
  @mkfs.fat -F 32 $(1)
endef

define make_disk_image
  $(if $(filter $(1),fat32), $(call make_disk_image_fat32,$(2)))
endef

define mk_pflash
  @RUSTFLAGS="" cargo build -p origin  --target riscv64gc-unknown-none-elf --release
  @rust-objcopy --binary-architecture=riscv64 --strip-all -O binary ./target/riscv64gc-unknown-none-elf/release/origin /tmp/origin.bin
  @printf "pfld\00\00\00\01" > /tmp/prefix.bin
  @printf "%08x" `stat -c "%s" /tmp/origin.bin` | xxd -r -ps > /tmp/size.bin
  @cat /tmp/prefix.bin /tmp/size.bin > /tmp/head.bin
  @dd if=/dev/zero of=./$(1) bs=1M count=32
  @dd if=/tmp/head.bin of=./$(1) conv=notrunc
  @dd if=/tmp/origin.bin of=./$(1) seek=16 obs=1 conv=notrunc
endef

define setup_disk
  $(call build_origin)
  $(call build_skernel2)

  @mkdir -p ./mnt
  @sudo mount $(1) ./mnt
  @sudo mkdir -p ./mnt/sbin
  @sudo cp /tmp/origin.bin ./mnt/sbin/origin
  @sudo cp /tmp/skernel2.bin ./mnt/sbin/skernel2

  @sudo umount ./mnt
  @sudo rm -rf mnt
endef

define build_origin
  @RUSTFLAGS="" cargo build -p origin  --target riscv64gc-unknown-none-elf --release
  @rust-objcopy --binary-architecture=riscv64 --strip-all -O binary ./target/riscv64gc-unknown-none-elf/release/origin /tmp/origin.bin
endef

define build_skernel2
  @RUSTFLAGS="" cargo build -p skernel2  --target riscv64gc-unknown-none-elf --release
  @rust-objcopy --binary-architecture=riscv64 --strip-all -O binary ./target/riscv64gc-unknown-none-elf/release/skernel2 /tmp/skernel2.bin
endef

# define build_mapfile
#   @make -C arceos/payload/mapfile_c
#   @cp ./payload/mapfile_c/mapfile /tmp/mapfile.bin
# endef
