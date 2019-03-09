using AutoMapper;
using FluentValidation.AspNetCore;
using MediatR;
using MediatR.Pipeline;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Diagnostics;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Metadata.Internal;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Pomelo.EntityFrameworkCore.MySql.Infrastructure;
using QCExtensions.Application.Comics.Commands.AddItemToComic;
using QCExtensions.Application.Comics.Models;
using QCExtensions.Application.Comics.Queries.GetComic;
using QCExtensions.Application.Infrastructure;
using QCExtensions.Application.Infrastructure.AutoMapper;
using QCExtensions.Application.Interfaces;
using QCExtensions.Application.Items.Models;
using QCExtensions.Domain.Entities;
using QCExtensions.Persistence;
using QCExtensions.Server.Extensions;
using QCExtensions.Server.Infrastructure;
using QCExtensions.Server.Infrastructure.EntityMaterializerSource;
using QCExtensions.Server.Infrastructure.Filters;
using QCExtensions.Server.Infrastructure.Services;
using QCExtensions.Server.Infrastructure.Services.Hosted;
using System;
using System.Net;
using System.Reflection;

namespace QCExtensions.Server
{
	public class Startup
	{
		private readonly IHostingEnvironment _env;
		private readonly IConfiguration _configuration;

		public Startup(IHostingEnvironment env, IConfiguration configuration)
		{
			_env = env;
			_configuration = configuration;
		}

		public void ConfigureServices(IServiceCollection services)
		{
			// Add AutoMapper
			services.AddAutoMapper(opts =>
			{
				opts.CreateMissingTypeMaps = true;
			}, new Assembly[] { typeof(AutoMapperProfile).GetTypeInfo().Assembly, typeof(Startup).GetTypeInfo().Assembly });

			// Add services
			services.AddScoped<ITokenValidator, TokenValidator>();
			services.AddScoped<IActionLogger, ActionLogger>();
			services.AddSingleton<IDateTime, DateTimeService>();

			// Add hosted services
			services.AddSingleton<INewsUpdater, NewsUpdater>();
			services.AddSingleton<Microsoft.Extensions.Hosting.IHostedService, BackgroundNewsUpdatingService>();
			services.AddSingleton<Microsoft.Extensions.Hosting.IHostedService, DailyComicUpdatingService>();

			// Add MediatR
			services.AddTransient(typeof(IPipelineBehavior<,>), typeof(RequestPreProcessorBehavior<,>));
			services.AddTransient(typeof(IPipelineBehavior<,>), typeof(RequestTokenValidationBehavior<,>));
			services.AddTransient(typeof(IPipelineBehavior<,>), typeof(RequestValidationBehavior<,>));
			services.AddTransient(typeof(IPipelineBehavior<,>), typeof(RequestPostProcessorBehavior<,>));
			services.AddMediatR(typeof(GetComicQuery).GetTypeInfo().Assembly);

			// Add Entity Framework
			services.AddDbContextPool<QCExtensionsDbContext>(
				options => options.UseMySql(_configuration.GetConnectionString("Default"),
					mysqlOptions =>
					{
						mysqlOptions.ServerVersion(
							new Version(10, 0, 36),
							ServerType.MariaDb);
					}
				).ReplaceService<IEntityMaterializerSource, DateTimeKindEntityMaterializerSource>()
			);
			services.AddScoped<DomainDbContext, QCExtensionsDbContext>();

			services.Configure<CookiePolicyOptions>(options =>
			{
				// This lambda determines whether user consent for non-essential cookies is needed for a given request.
				options.CheckConsentNeeded = context => true;
				options.MinimumSameSitePolicy = SameSiteMode.None;
			});

			// Customise default API behavour
			services.Configure<ApiBehaviorOptions>(options =>
			{
				options.SuppressModelStateInvalidFilter = true;
			});

			services
				.AddMemoryCache()
				.AddMvc(o =>
				{
					o.Filters.Add(typeof(VersionLoggingFilter));
				})
				.AddJsonOptions(o =>
				{
					var fluentContractResolver = new FluentContractResolver();

					fluentContractResolver.ForType<ComicDto>()
						.IgnoreNull(c => c.EditorData)
						.IgnoreNull(c => c.AllItems);

					fluentContractResolver.ForType<ItemWithNavigationDataDto>()
						.Ignore(i => i.Count);

					o.SerializerSettings.ContractResolver = fluentContractResolver;
				})
				.SetCompatibilityVersion(CompatibilityVersion.Version_2_2)
				.AddFluentValidation(fv => fv.RegisterValidatorsFromAssemblyContaining<AddItemToComicCommandValidator>());
		}

		public void Configure(IApplicationBuilder app, IHostingEnvironment env)
		{
			if (env.IsDevelopment())
			{
				app.UseDeveloperExceptionPage();
			}

			app.UseExceptionHandler(
				builder =>
				{
					builder.Run(
						async context =>
						{
							context.Response.StatusCode = (int)HttpStatusCode.InternalServerError;
							context.Response.Headers.Add("Access-Control-Allow-Origin", "*");

							var error = context.Features.Get<IExceptionHandlerFeature>();
							if (error != null)
							{
								context.Response.AddApplicationError(error.Error.Message);
								await context.Response.WriteAsync(error.Error.Message).ConfigureAwait(false);
							}
						}
					);
				}
			);

			app
				.UseDefaultFiles()
				.UseStaticFiles()
				.UseCookiePolicy()
				.UseCors(builder =>
					builder.AllowAnyOrigin()
						.AllowAnyMethod()
						.AllowAnyHeader()
						.AllowCredentials()
				)
				.UseMvc(routes =>
				{
					routes.MapRoute("default", "{controller=Home}/{action=Index}/{id?}");

					routes.MapRoute("Spa", "{*url}", defaults: new { controller = "Home", action = "Spa" });
				});
		}
	}
}
