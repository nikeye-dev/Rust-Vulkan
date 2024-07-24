#define PI 3.1415926535897932
#define CM_TO_SKY_UNIT 0.00001f

struct ViewState
{
	float3 WorldCameraOrigin;

	float3 AtmosphereLightDirection;
	float3 AtmosphereLightIlluminanceOuterSpace;
};

uniform ViewState ResolvedView;

struct FSampleData
{
	float3 PlanetPos;
	float PlanetRadius;
	float AtmosphereThickness;
	half SampleCount;
	half SampleCountLight;
	float Scale;

	float3 LightDir;
	float3 LightIntensity;
};

struct FMedium
{
	float ScaleHeightR;
	float ScaleHeightM;
	
	//Rayleigh
	float3 ScatteringR;
	float3 AbsorptionR;
	float3 ExtinctionR;

	//Mie
	float3 ScatteringM;
	float3 AbsorptionM;
	float3 ExtinctionM;

	void InitCoefficients(const float3 ScatteringRay)
	{
		ScatteringR = ScatteringRay;
		AbsorptionR = 0;
		ExtinctionR = ScatteringR + AbsorptionR;
		//
		// ScatteringM = ScatteringMie;
		// AbsorptionM = AbsorptionMie;
		// ExtinctionM = ScatteringM + AbsorptionM;
	}
};

struct FAtmosphere
{
	FSampleData Sample;
	FMedium Medium;

	float3 SingleScattering(const float3 WorldPos, float3 ViewPos);

	float2 RayIntersectSphere(float3 RayOrigin, float3 RayDirection, float4 Sphere)
	{
		float3 LocalPosition = RayOrigin - Sphere.xyz;
		float LocalPositionSqr = dot(LocalPosition, LocalPosition);

		float3 QuadraticCoef;
		QuadraticCoef.x = dot(RayDirection, RayDirection);
		QuadraticCoef.y = 2 * dot(RayDirection, LocalPosition);
		QuadraticCoef.z = LocalPositionSqr - Sphere.w * Sphere.w;

		float Discriminant = QuadraticCoef.y * QuadraticCoef.y - 4 * QuadraticCoef.x * QuadraticCoef.z;

		float2 Intersections = -1;

		// Only continue if the ray intersects the sphere
		if (Discriminant >= 0)
		{
			float SqrtDiscriminant = sqrt(Discriminant);
			Intersections = (-QuadraticCoef.y + float2(-1, 1) * SqrtDiscriminant) / (2 * QuadraticCoef.x);
		}

		return Intersections;
	}

	// Theta - angle between light direction and view direction
	float PhaseRayleigh(const float CosTheta)
	{
		return (3.0f / (16.0f * PI)) * (1.0f + CosTheta * CosTheta);
	}

	//ToDo: This is redundant when we have input struct outside Unreal
	float3 Render(const float3 WorldPos, const float3 PlanetPos,
		const float PlanetRadius, const float AtmosphereThickness,
		const half SampleCount, const half SampleCountLight,
		const float ScaleHeightRay, const float3 ScatteringRay,
		const float UnitScale = 1.0f)
	{
		Sample.Scale = UnitScale;

		Sample.PlanetPos = PlanetPos;
		Sample.PlanetRadius = PlanetRadius;
		Sample.AtmosphereThickness = AtmosphereThickness;
		Sample.SampleCount = SampleCount;
		Sample.SampleCountLight = SampleCountLight;

		Sample.LightDir = ResolvedView.AtmosphereLightDirection.xyz;
		Sample.LightIntensity = ResolvedView.AtmosphereLightIlluminanceOuterSpace.rgb;

		Medium.ScaleHeightR = ScaleHeightRay;
		Medium.InitCoefficients(ScatteringRay);
		
		return SingleScattering(WorldPos, ResolvedView.WorldCameraOrigin);
	}

	float3 SingleScattering(const float3 WorldPos, float3 ViewPos)
	{
		const float3 ViewDir = normalize(WorldPos - ViewPos);
		
		ViewPos = (ViewPos - Sample.PlanetPos) * CM_TO_SKY_UNIT * Sample.Scale;

		const float4 Atmosphere = float4(0, 0, 0, Sample.PlanetRadius + Sample.AtmosphereThickness);
		
		float2 P = RayIntersectSphere(ViewPos, ViewDir, Atmosphere);
		if(P.x < 0 && P.y < 0)
			return 0;

		float2 PPlanet = RayIntersectSphere(ViewPos, ViewDir, float4(0, 0, 0, Sample.PlanetRadius));
		if(PPlanet.x > 0)
			P.y = PPlanet.x;
		
		P.x = max(P.x, 0.0f);
		P.y = min(P.y, 9000000.0f);

		//Accumulated light
		float3 A = 0;
		
		float OpticalDepth = 0;
		float CurrentSample = P.x;
		// Sample size
		const float Ds = (P.y - P.x) / float(Sample.SampleCount);

		for(int i = 0; i < Sample.SampleCount; i++)
		{
			//Sample point X
			const float3 X = ViewPos + ViewDir * (CurrentSample + Ds * 0.5);

			//Height/Altitude
			const float H = length(X) - Sample.PlanetRadius;

			const float DensityX = exp(-H / Medium.ScaleHeightR) * Ds;
			OpticalDepth += DensityX;

			//Light transmittance
			float2 PLight = RayIntersectSphere(X, Sample.LightDir, Atmosphere);
			
			float CurrentSampleLight = 0;
			float OpticalDepthLight = 0;

			const float DsLight = PLight.y / float(Sample.SampleCountLight);

			bool Underground = false;
			
			for(int j = 0; j < Sample.SampleCountLight; j++)
			{
				//ToDo: Unify float4. Light dir negated here?
				const float3 XLight = X + Sample.LightDir * (CurrentSampleLight + DsLight * 0.5);
				const float HLight = length(XLight) - Sample.PlanetRadius;
				if(HLight < 0)
				{
					Underground = true;
					break;
				}
				
				const float DensityLight = exp(-HLight / Medium.ScaleHeightR) * DsLight;

				OpticalDepthLight += DensityLight;
				CurrentSampleLight += DsLight;
			}
			//============ Light

			if(!Underground)
			{
				const float3 Transmittance = exp(-Medium.ExtinctionR * (OpticalDepth + OpticalDepthLight));
				A += Transmittance * DensityX;
			}
			
			CurrentSample += Ds;
		}

		const float CosTheta = dot(Sample.LightDir, ViewDir);
		const float PhaseR = PhaseRayleigh(CosTheta);
		float3 L = Sample.LightIntensity * (PhaseR * Medium.ScatteringR * A);
		
		return L;
	}
	
};

struct PS_OUTPUT
{
    float4 Color[4] : COLOR0;
};

PS_OUTPUT main()
{
	PS_OUTPUT result;

	//tmp
	float3 WorldPos = float3(0, 0, 0);
	float3 PlanetPos = float3(0, 0, 0);
	float PlanetRadius = 100;
	float AtmosphereThickness = 1;
	half SampleCount = 5;
	half SampleCountLight = 5;
	float ScaleHeightRay = 10;
	float ScatteringRay = float3(0.5, 0.5, 0.5);

	FAtmosphere atmosphere;
	result.Color = float4(atmosphere.Render(WorldPos, PlanetPos, PlanetRadius, AtmosphereThickness, SampleCount, SampleCountLight, ScaleHeightRay, ScatteringRay), 1.0);

	return result;
}
