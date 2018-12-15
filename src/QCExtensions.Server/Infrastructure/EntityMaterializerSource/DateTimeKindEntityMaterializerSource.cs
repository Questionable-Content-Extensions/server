using System;
using System.Linq.Expressions;
using System.Reflection;
using Microsoft.EntityFrameworkCore.Metadata;

namespace QCExtensions.Server.Infrastructure.EntityMaterializerSource
{
	public class DateTimeKindMapper
	{
		public static DateTime Normalize(DateTime value)
			=> DateTime.SpecifyKind(value, DateTimeKind.Utc);

		public static DateTime? NormalizeNullable(DateTime? value)
			=> value.HasValue ? DateTime.SpecifyKind(value.Value, DateTimeKind.Utc) : (DateTime?)null;

		public static object NormalizeObject(object value)
			=> value is DateTime dateTime ? Normalize(dateTime) : value;
	}

	public class DateTimeKindEntityMaterializerSource : Microsoft.EntityFrameworkCore.Metadata.Internal.EntityMaterializerSource
	{
		private static readonly MethodInfo _normalizeMethod =
			typeof(DateTimeKindMapper).GetTypeInfo().GetMethod(nameof(DateTimeKindMapper.Normalize));

		private static readonly MethodInfo _normalizeNullableMethod =
			typeof(DateTimeKindMapper).GetTypeInfo().GetMethod(nameof(DateTimeKindMapper.NormalizeNullable));

		private static readonly MethodInfo _normalizeObjectMethod =
			typeof(DateTimeKindMapper).GetTypeInfo().GetMethod(nameof(DateTimeKindMapper.NormalizeObject));

		public override Expression CreateReadValueExpression(Expression valueBuffer, Type type, int index, IPropertyBase property = null)
		{
			if (type == typeof(DateTime))
			{
				return Expression.Call(
					_normalizeMethod,
					base.CreateReadValueExpression(valueBuffer, type, index, property));
			}

			if (type == typeof(DateTime?))
			{
				return Expression.Call(
					_normalizeNullableMethod,
					base.CreateReadValueExpression(valueBuffer, type, index, property));
			}

			return base.CreateReadValueExpression(valueBuffer, type, index, property);
		}

		public override Expression CreateReadValueCallExpression(Expression valueBuffer, int index)
		{
			var readValueCallExpression = base.CreateReadValueCallExpression(valueBuffer, index);
			if (readValueCallExpression.Type == typeof(DateTime))
			{
				return Expression.Call(
					_normalizeMethod,
					readValueCallExpression);
			}

			if (readValueCallExpression.Type == typeof(DateTime?))
			{
				return Expression.Call(
					_normalizeNullableMethod,
					readValueCallExpression);
			}

			if (readValueCallExpression.Type == typeof(object))
			{
				return Expression.Call(
					_normalizeObjectMethod,
					readValueCallExpression);
			}

			return readValueCallExpression;
		}
	}
}