import React, { useRef } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import * as THREE from 'three';
import './LiquidOrbit.scss';

const vertexShader = `
  varying vec2 vUv;
  void main() {
    vUv = uv;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  }
`;

const fragmentShader = `
    uniform float u_time;
    uniform vec3 u_color;
    varying vec2 vUv;

    // 2D Random
    float random (in vec2 st) {
        return fract(sin(dot(st.xy,
                             vec2(12.9898,78.233)))
                     * 43758.5453123);
    }

    // 2D Noise based on Morgan McGuire @morgan3d
    // https://www.shadertoy.com/view/4dS3Wd
    float noise (in vec2 st) {
        vec2 i = floor(st);
        vec2 f = fract(st);

        // Four corners in 2D of a tile
        float a = random(i);
        float b = random(i + vec2(1.0, 0.0));
        float c = random(i + vec2(0.0, 1.0));
        float d = random(i + vec2(1.0, 1.0));

        vec2 u = f * f * (3.0 - 2.0 * f);

        return mix(a, b, u.x) +
                (c - a)* u.y * (1.0 - u.x) +
                (d - b) * u.x * u.y;
    }

    void main() {
      vec2 p = vUv * 2.0 - 1.0;
      float n = noise(p * 4.0 + u_time * 0.5);
      float ring = smoothstep(0.4, 0.45, length(p));
      float inner = smoothstep(0.3, 0.35, length(p));
      
      float color = ring - inner;
      color += n * 0.1;

      gl_FragColor = vec4(u_color * color, color);
    }
`;

const LiquidOrbitMaterial = new THREE.ShaderMaterial({
  uniforms: {
    u_time: { value: 0 },
    u_color: { value: new THREE.Color('#ff4800') },
  },
  vertexShader,
  fragmentShader,
  transparent: true,
});

const OrbitSphere = () => {
  const materialRef = useRef<THREE.ShaderMaterial>();
  useFrame(({ clock }) => {
    if (materialRef.current) {
      materialRef.current.uniforms.u_time.value = clock.getElapsedTime();
    }
  });

  return (
    <mesh>
      <sphereGeometry args={[1, 32, 32]} />
      <primitive object={LiquidOrbitMaterial} ref={materialRef} />
    </mesh>
  );
};

export const LiquidOrbit: React.FC = () => {
  return (
    <div className="liquid-orbit-container">
      <Canvas>
        <OrbitSphere />
      </Canvas>
    </div>
  );
};
