// Copyright (C) 2025 Palantir
// This program is free software; you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation; either version 2 of the License, or (at your option)
// any later version.
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for
// more details.
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc., 59
// Temple Place, Suite 330, Boston, MA 02111-1307 USA
// sbhatia@palantir.com
// "CLASSPATH" EXCEPTION TO THE GPL
// Certain source files distributed by Oracle America and/or its affiliates are
// subject to the following clarification and special exception to the GPL, but
// only where Oracle has expressly included in the particular source file's header
// the words "Oracle designates this particular file as subject to the "Classpath"
// exception as provided by Oracle in the LICENSE file that accompanied this code."
// Linking this library statically or dynamically with other modules is making
// a combined work based on this library.  Thus, the terms and conditions of
// the GNU General Public License cover the whole combination.
// As a special exception, the copyright holders of this library give you
// permission to link this library with independent modules to produce an
// executable, regardless of the license terms of these independent modules,
// and to copy and distribute the resulting executable under terms of your
// choice, provided that you also meet, for each linked independent module,
// the terms and conditions of the license of that module.  An independent
// module is a module which is not derived from or based on this library.  If
// you modify this library, you may extend this exception to your version of
// the library, but you are not obligated to do so.  If you do not wish to do
// so, delete this exception statement from your version. "

import java.io.BufferedWriter;
import java.io.FileWriter;
import java.io.IOException;
import java.time.Instant;
import java.time.LocalDate;
import java.time.LocalTime;
import java.time.ZoneId;
import java.time.ZonedDateTime;
import java.time.chrono.IsoChronology;
import java.time.format.DateTimeFormatter;
import java.time.temporal.TemporalAccessor;
import java.time.temporal.TemporalQueries;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Locale;
import java.util.Random;

public class DateTimeFormatterCSVGenerator {

    // Define a set of common patterns to test
    private static final List<String> PATTERNS = List.of(
        "yyyy-MM-dd'T'HH:mm:ss.SSSSSS G", // ISO-8601 without timezone with micros and era
        "yyyy-MM-dd'T'HH:mm:ss.SSSSSS", // ISO-8601 without timezone with micros
        "yyyy-MM-dd HH:mm:ss.SSSSSS", // Standard datetime with micros
        "yyyy-MM-dd", // ISO date
        "EEEE, MMMM d, yyyy", // Full text date
        "MM/dd/yyyy", // US date format
        "dd/MM/yyyy", // European date format
        "yyyy/MM/dd", // Year first date format
        "MMMM d, yyyy", // Month name date
        "d MMMM yyyy", // Day first with month name
        "yyyyMMddHHmmss", // Compact datetime
        "yyyy-DDD", // Year with day of year
        "YYYY-'W'ww-e", // ISO week date
        "yyyy-'W'ww-e", // ISO week date but with regular year
        "uuuu-MM-dd", // Year format
        "yyyy.MM.dd 'at' HH:mm:ss.SSSSSS" // Full date and time with micros
    );

    // Fixed settings
    private static final ZoneId ZONE_ID = ZoneId.of("UTC");
    private static final Locale LOCALE = Locale.US;
    private static final long SEED = 42L; // Fixed seed for reproducible testing
    private static final Random RANDOM = new Random(SEED);

    public static void main(String[] args) {
        // Set default locale to US for the entire application
        Locale.setDefault(LOCALE);

        int numTestCases = 20000;

        // Output to file
        try {
            generateAndWriteTestData(numTestCases, "datetime_test_data.csv");
            System.out.println(
                "Successfully generated test cases for " +
                numTestCases +
                " timestamps with " +
                PATTERNS.size() +
                " patterns each in CSV format."
            );
        } catch (IOException e) {
            System.err.println("Error writing test data: " + e.getMessage());
        }
    }

    private static void generateAndWriteTestData(
        int numTestCases,
        String fileName
    ) throws IOException {
        // Create a list to hold all data rows
        List<CsvDataRow> dataRows = new ArrayList<>();

        long minEpochMicros = -125923200000000000L; // Roughly 2000 BC
        long maxEpochMicros = 64060588800000000L; // Roughly 4000 AD
        // Generate test data
        for (int i = 0; i < numTestCases; i++) {
            // Generate a random timestamp between 2000 BC and 4000 AD using fixed seed
            long timestamp =
                minEpochMicros +
                (long) ((maxEpochMicros - minEpochMicros) *
                    RANDOM.nextDouble());

            // Convert timestamp to ZonedDateTime with UTC zone
            // Convert microseconds to seconds and nanoseconds
            long epochSeconds = timestamp / 1_000_000;
            int nanos = (int) ((timestamp % 1_000_000) * 1000);

            ZonedDateTime dateTime;
            try {
                dateTime = Instant.ofEpochSecond(epochSeconds, nanos).atZone(
                    ZONE_ID
                );
            } catch (Exception e) {
                // Skip invalid timestamps
                System.err.println(
                    "Invalid timestamp: " +
                    timestamp +
                    ", error: " +
                    e.getMessage()
                );
                continue;
            }

            // For each pattern, create a row
            for (String pattern : PATTERNS) {
                String formatted = "";
                String recoveredTimestamp = "";

                try {
                    // Format the timestamp with the pattern
                    DateTimeFormatter formatter = DateTimeFormatter.ofPattern(
                        pattern,
                        LOCALE
                    )
                        .withChronology(IsoChronology.INSTANCE)
                        .withZone(ZONE_ID);

                    formatted = dateTime.format(formatter);

                    // Attempt to parse the formatted string back to a timestamp
                    try {
                        TemporalAccessor parsed = formatter.parse(formatted);

                        // Handle different types of date-time information
                        ZonedDateTime parsedDateTime = null;

                        // Check what kind of temporal information we have
                        LocalDate date = parsed.query(
                            TemporalQueries.localDate()
                        );
                        LocalTime time = parsed.query(
                            TemporalQueries.localTime()
                        );
                        ZoneId zone = parsed.query(TemporalQueries.zone());

                        if (date != null) {
                            if (time != null) {
                                // We have date and time
                                if (zone != null) {
                                    // Complete ZonedDateTime available
                                    parsedDateTime = ZonedDateTime.of(
                                        date,
                                        time,
                                        zone
                                    );
                                } else {
                                    // No zone, use the default
                                    parsedDateTime = ZonedDateTime.of(
                                        date,
                                        time,
                                        ZONE_ID
                                    );
                                }
                            } else {
                                // Only date, no time (use midnight)
                                parsedDateTime = date.atStartOfDay(ZONE_ID);
                            }

                            // Calculate the recovered timestamp
                            long recoveredMicros =
                                parsedDateTime.toInstant().getEpochSecond() *
                                    1_000_000 +
                                parsedDateTime.getNano() / 1000;
                            recoveredTimestamp = String.valueOf(
                                recoveredMicros
                            );
                        }
                        // If we couldn't extract a date, leave recoveredTimestamp as empty

                    } catch (Exception e) {
                        // If parsing fails, leave as empty string
                        // But log if it's unexpected
                        if (pattern.equals("yyyy-MM-dd")) {
                            System.err.println(
                                "Failed to parse with pattern " +
                                pattern +
                                ": " +
                                e.getMessage() +
                                " for formatted value: " +
                                formatted
                            );
                        }
                    }
                } catch (Exception e) {
                    // If formatting fails, leave as empty string and log
                    System.err.println(
                        "Error processing timestamp " +
                        timestamp +
                        " with pattern " +
                        pattern +
                        ": " +
                        e.getMessage()
                    );
                }

                // Add the row to our data list
                dataRows.add(
                    new CsvDataRow(
                        timestamp,
                        pattern,
                        formatted,
                        recoveredTimestamp
                    )
                );
            }
        }

        // Sort the data rows by timestamp
        dataRows.sort(Comparator.comparingLong(CsvDataRow::timestamp));

        // Now write all data to file
        try (
            BufferedWriter writer = new BufferedWriter(new FileWriter(fileName))
        ) {
            // Write header
            writer.write(
                "original_timestamp,pattern,formatted,recovered_timestamp"
            );
            writer.newLine();

            // Write sorted rows
            for (CsvDataRow row : dataRows) {
                writer.write(
                    row.timestamp() +
                    ",\"" +
                    escapeCSV(row.pattern()) +
                    "\",\"" +
                    escapeCSV(row.formatted()) +
                    "\"," +
                    row.recoveredTimestamp()
                );
                writer.newLine();
            }
        }
    }

    // Record to store CSV row data for sorting
    private record CsvDataRow(
        long timestamp,
        String pattern,
        String formatted,
        String recoveredTimestamp
    ) {}

    private static String escapeCSV(String input) {
        if (input == null) return "";
        // Double any quotes and wrap in quotes if needed
        return input.replace("\"", "\"\"");
    }
}
