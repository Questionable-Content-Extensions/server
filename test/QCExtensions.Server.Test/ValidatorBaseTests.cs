using System;
using FluentValidation;
using FluentValidation.Results;
using Xunit;

namespace QCExtensions.Server.Test
{
	public abstract class ValidatorBaseTests<TModel, TValidator> where TValidator : IValidator<TModel>, new()
	{
		protected abstract TModel GetValidViewModel();

		protected ValidationResult Validate(TModel model)
		{
			var sut = new TValidator();
			var result = sut.Validate(model);
			return result;
		}

		protected void MissingFieldShouldGiveValidationError(Action<TModel> modifyValidModel, string isValidErrorMessage, string expectedValidationMessage)
		{
			var model = GetValidViewModel();
			modifyValidModel(model);
			var result = Validate(model);

			Assert.False(result.IsValid, isValidErrorMessage);
			Assert.Contains(result.Errors, e => e.ErrorMessage == expectedValidationMessage);
		}

		[Fact]
		public void ValidModelShouldNotGiveValidationError()
		{
			var model = GetValidViewModel();
			var result = Validate(model);
			Assert.True(result.IsValid, "Valid model should not give validation error");
		}
	}
}