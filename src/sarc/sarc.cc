#include "roead/include/sarc.h"
#include "roead/src/lib.rs.h"
#include "rust/cxx.h"
#include <iostream>
#include <memory>
#include <nonstd/span.h>
#include <oead/sarc.h>
#include <stdexcept>
#include <string_view>

Sarc::Sarc(const rust::Slice<const uint8_t> data)
    : inner({data.data(), data.size()}) {}

std::unique_ptr<Sarc> sarc_from_binary(const rust::Slice<const uint8_t> data) {
  return std::unique_ptr<Sarc>(new Sarc(data));
}

rust::Slice<const uint8_t> Sarc::get_file_data(const rust::Str name) const {
  auto maybe_file = inner.GetFile(std::string_view(name.data(), name.size()));
  if (!maybe_file.has_value()) {
    throw std::runtime_error("File not found in SARC");
  }
  auto file = maybe_file.value();
  return rust::Slice<const uint8_t>(file.data.data(), file.data.size());
}

rust::Slice<const uint8_t> Sarc::idx_file_data(const uint16_t idx) const {
  auto file = inner.GetFile(idx);
  return rust::Slice(file.data.data(), file.data.size());
}

rust::Str Sarc::idx_file_name(const uint16_t idx) const {
  auto file = inner.GetFile(idx);
  return rust::Str(file.name.data(), file.name.size());
}

uint16_t Sarc::num_files() const { return inner.GetNumFiles(); }

uint32_t Sarc::get_offset() const { return inner.GetDataOffset(); }

size_t Sarc::guess_align() const { return inner.GuessMinAlignment(); }

bool Sarc::big_endian() const {
  return (inner.GetEndianness() == oead::util::Endianness::Big);
}

bool Sarc::files_eq(const Sarc &other) const {
  return this->inner.AreFilesEqual(other.inner);
}

SarcWriteResult WriteSarc(const RsSarcWriter& rs_writer, bool big_endian, bool legacy, uint8_t align) {
  auto writer = new oead::SarcWriter(
    big_endian ? oead::util::Endianness::Big : oead::util::Endianness::Little,
    legacy ? oead::SarcWriter::Mode::Legacy : oead::SarcWriter::Mode::New);
  writer->SetMinAlignment(align);
  for (size_t i = 0; i < rs_writer.len(); i++) {
    auto name = rs_writer.get_file_by_index(i);
    auto path = std::string(name.data(), name.size());
    auto data = rs_writer.get_data_by_index(i);
    std::vector<uint8_t> vec(data.begin(), data.end());
    writer->m_files[path] = vec;
  }
  auto result = writer->Write();
  SarcWriteResult res{};
  res.alignment = result.first;
  res.data = std::make_unique<std::vector<uint8_t>>(result.second);
  return res;
}
