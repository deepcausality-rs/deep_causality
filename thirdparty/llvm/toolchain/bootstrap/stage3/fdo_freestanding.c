typedef unsigned int fdo_value;

static __attribute__((noinline)) fdo_value rotate_left(fdo_value value,
                                                       fdo_value amount) {
  amount &= 31;
  if (amount == 0)
    return value;
  return (value << amount) | (value >> (32 - amount));
}

fdo_value llvm_fdo_freestanding(fdo_value value) {
  for (fdo_value index = 1; index != 17; ++index) {
    if (value & index)
      value = rotate_left(value ^ (index * 0x9e3779b9U), index);
    else
      value += rotate_left(index, value & 15);
  }
  return value;
}
