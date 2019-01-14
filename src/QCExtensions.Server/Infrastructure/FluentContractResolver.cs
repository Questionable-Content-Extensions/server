using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;
using System;
using System.Collections.Generic;
using System.Linq.Expressions;
using System.Reflection;

namespace QCExtensions.Server.Infrastructure
{
	public class FluentContractResolver : CamelCasePropertyNamesContractResolver
	{
		public class FluentContractResolverTypeBuilder<T>
		{
			private readonly FluentContractResolverTypeConfiguration configuration;

			public FluentContractResolverTypeBuilder(FluentContractResolverTypeConfiguration configuration)
			{
				this.configuration = configuration;
			}

			private static string GetReturnedPropertyName<TR>(Expression<Func<T, TR>> propertyLambda)
			{
				var member = propertyLambda.Body as MemberExpression;
				var memberPropertyInfo = member?.Member as PropertyInfo;
				return memberPropertyInfo?.Name;
			}

			internal FluentContractResolverTypeBuilder<T> IgnoreNull<TR>(Expression<Func<T, TR>> propertyLambda)
			{
				var propertyName = GetReturnedPropertyName(propertyLambda);
				configuration.IgnoredNullProperties.Add(propertyName);

				return this;
			}

			internal FluentContractResolverTypeBuilder<T> Ignore<TR>(Expression<Func<T, TR>> propertyLambda)
			{
				var propertyName = GetReturnedPropertyName(propertyLambda);
				configuration.IgnoredProperties.Add(propertyName);

				return this;
			}
		}

		public class FluentContractResolverTypeConfiguration
		{
			public List<string> IgnoredProperties = new List<string>();
			public List<string> IgnoredNullProperties = new List<string>();
		}

		private Dictionary<Type, FluentContractResolverTypeConfiguration> configurations = new Dictionary<Type, FluentContractResolverTypeConfiguration>();

		public FluentContractResolverTypeBuilder<T> ForType<T>()
		{
			configurations.TryGetValue(typeof(T), out var configuration);
			if (configuration == null)
			{
				configuration = new FluentContractResolverTypeConfiguration();
				configurations.Add(typeof(T), configuration);
			}
			return new FluentContractResolverTypeBuilder<T>(configuration);
		}

		protected override JsonProperty CreateProperty(MemberInfo member, MemberSerialization memberSerialization)
		{
			var property = base.CreateProperty(member, memberSerialization);
			configurations.TryGetValue(member.DeclaringType, out var configuration);
			if (configuration == null)
			{
				return property;
			}

			if (configuration.IgnoredNullProperties.Contains(member.Name))
			{
				property.NullValueHandling = NullValueHandling.Ignore;
			}
			if (configuration.IgnoredProperties.Contains(member.Name))
			{
				property.Ignored = true;
			}

			return property;
		}
	}
}
