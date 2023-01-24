package org.osm2streets;

import org.osm2streets.LatLon;

import java.io.InputStream;
import java.net.URL;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.List;

public class StreetNetwork {
	long pointer;

	StreetNetwork(long pointer) {
		this.pointer = pointer;
	}

	static {
		try {
			String libName = "libosm2streets_java.so";
			URL libUrl = StreetNetwork.class.getResource("/" + libName);
			File tmpDir = Files.createTempDirectory("jni-native-libs").toFile();
			tmpDir.deleteOnExit();
			File nativeLibTmpFile = new File(tmpDir, libName);
			nativeLibTmpFile.deleteOnExit();
			try (InputStream in = libUrl.openStream()) {
				Files.copy(in, nativeLibTmpFile.toPath());
			}
			System.load(nativeLibTmpFile.getAbsolutePath());
		} catch (IOException e) {
			System.err.println("Failed to extract/load native lib: " + e.toString());
		}
	}

	public static native StreetNetwork create(String osmXmlInput);

	public native List<List<LatLon>> getRoadSurface();

	public native String toLanePolygonsGeojson();

	public native String toLaneMarkingsGeojson();

	public static void main(String[] args) throws Exception {
		String osmXmlInput = new String(Files.readAllBytes(Paths.get("../tests/src/aurora_sausage_link/input.osm")));
		StreetNetwork network = create(osmXmlInput);
		System.out.println(network.toLanePolygonsGeojson());
		System.out.println(network.toLaneMarkingsGeojson());
	}
}
