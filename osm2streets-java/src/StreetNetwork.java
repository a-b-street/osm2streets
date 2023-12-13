package org.osm2streets;

import org.osm2streets.LatLon;
import org.osm2streets.Surface;
import org.osm2streets.PaintArea;

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

	public static native StreetNetwork create(byte[] osmInput);

	public native List<Surface> getSurfaces();

	public native List<PaintArea> getPaintAreas();

	public static void main(String[] args) throws Exception {
		byte[] osmInput = Files.readAllBytes(Paths.get("../tests/src/aurora_sausage_link/input.osm"));
		StreetNetwork network = create(osmInput);
		System.out.println(network.getSurfaces());
		System.out.println(network.getPaintAreas());
	}
}
