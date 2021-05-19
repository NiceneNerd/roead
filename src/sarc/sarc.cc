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

std::unique_ptr<SarcWriter> NewSarcWriter(bool big_endian, bool legacy) {
  return std::unique_ptr<SarcWriter>(new SarcWriter(
      big_endian ? oead::util::Endianness::Big : oead::util::Endianness::Little,
      legacy ? oead::SarcWriter::Mode::Legacy : oead::SarcWriter::Mode::New));
}

void SarcWriter::SetEndianness(bool big_endian) {
  oead::SarcWriter::SetEndianness(big_endian ? oead::util::Endianness::Big
                                             : oead::util::Endianness::Little);
}

void SarcWriter::SetMode(bool legacy) {
  oead::SarcWriter::SetMode(legacy ? oead::SarcWriter::Mode::Legacy
                                   : oead::SarcWriter::Mode::New);
}

void SarcWriter::SetFile(rust::Str name, rust::Vec<uint8_t> data) {
  auto path = std::string(name.data(), name.size());
  std::vector<uint8_t> vec;
  std::move(data.begin(), data.end(), std::back_inserter(vec));
  m_files[path] = vec;
}

bool SarcWriter::DelFile(rust::Str name) {
  return m_files.erase(std::string(name.data(), name.size())) > 0;
}

bool SarcWriter::FilesEqual(const SarcWriter &other) const
{
  return m_files == other.m_files;
}

size_t SarcWriter::NumFiles() const {
  return m_files.size();
}

std::unique_ptr<SarcWriter> WriterFromSarc(const Sarc& archive) {
  SarcWriter writer = SarcWriter(archive.inner.GetEndianness(), oead::SarcWriter::Mode::New);
  writer.SetMinAlignment(archive.inner.GuessMinAlignment());
  writer.m_files.reserve(archive.inner.GetNumFiles());
  for (const oead::Sarc::File& file : archive.inner.GetFiles()) {
    writer.m_files.emplace(std::string(file.name),
                           std::vector<u8>(file.data.begin(), file.data.end()));
  }
  return std::make_unique<SarcWriter>(writer);
}

SarcWriteResult SarcWriter::Write() {
  auto result = oead::SarcWriter::Write();
  rust::Vec<uint8_t> vec;
  std::move(result.second.begin(), result.second.end(),
            std::back_inserter(vec));
  SarcWriteResult res{};
  res.alignment = result.first;
  res.data = vec;
  return res;
}
